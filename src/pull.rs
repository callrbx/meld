use std::{
    fs,
    io::{BufRead, BufReader},
};

use crate::Args;
use libmeld::{hash_contents, hash_path, mapper, Bin, Error};
use log::{debug, error, info, warn};
use structopt::StructOpt;

// Define Module Arguments
#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(
        short = "r",
        long = "recent",
        help = "if matching tag not found, pull most recent"
    )]
    pub(crate) recent: bool,

    #[structopt(short = "t", long = "tag", default_value = "", help = "config tag")]
    pub(crate) tag: String,
    #[structopt(short = "v", long = "version", default_value = "0", help = "version")]
    pub(crate) version: u32,
    #[structopt(help = "config file/folder to pull")]
    pub(crate) config_path: String,
}

/// Function to actually copy configs from the blobs directory
fn copy_file(
    path: &String,
    blob_name: &String,
    blobs_dir: &String,
    version: u32,
) -> Result<(), Error> {
    // construct blob and version path for config
    let blob_ver_path = format!("{}/{}/{}", blobs_dir, blob_name, version);

    debug!("copy {} -> {}", blob_ver_path, path);

    let real_path = mapper::map_to_real_path(path)?;

    return match fs::copy(blob_ver_path, real_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::IOError { msg: e.to_string() }),
    };
}

/// Pull single file from the DB
fn pull_file(
    bin: &Bin,
    blob: &String,
    tag: &String,
    recent: bool,
    version: u32,
) -> Result<(), libmeld::Error> {
    let config_versions = bin.db.get_versions(blob)?;

    let map_path = match bin.db.get_mapped_path(&blob)? {
        Some(s) => s,
        None => {
            return Err(Error::FileNotFound {
                msg: blob.to_string(),
            })
        }
    };

    let path = mapper::map_to_real_path(&map_path)?;

    let cur_hash = if libmeld::exists(&path) {
        Some(hash_contents(&path)?).unwrap()
    } else {
        "".to_string()
    };

    let mut found_ver = 0;
    let mut max_ver = 0;
    let mut pull_ver = "";

    for (k, v) in &config_versions {
        if tag != "" && tag == &v.tag {
            debug!("Found matching tag: \"{}\" - {}", tag, k);
            found_ver = v.ver;
            pull_ver = k;
            break;
        }
        if version != 0 && version == v.ver {
            debug!("Found matching version: \"{}\" - {}", version, k);
            found_ver = v.ver;
            pull_ver = k;
            break;
        }
        if v.ver > max_ver {
            max_ver = v.ver;
            pull_ver = k;
        }
    }

    if tag != "" && version != 0 && found_ver == 0 {
        warn!("Failed to find specified matching version");
        if recent {
            info!("Updating to most recent version");
        } else {
            error!("Run with -r to override to most recent");
            return Err(Error::TagNotFound {
                msg: tag.to_string(),
            });
        }
    }

    let pulled_version = match config_versions.get(pull_ver) {
        Some(v) => v,
        None => {
            error!("Something failed in matching versions");
            return Err(Error::SomethingFailed);
        }
    };

    info!("Pulling version {}", pulled_version.ver);

    // check if update needed
    let update_needed = pulled_version.data_hash != cur_hash;

    if update_needed {
        info!("Updating config");
        if pulled_version.data_hash == "DIR" {
            info!("creating new dir");
            if fs::create_dir(path).is_err() {
                return Err(Error::IOError {
                    msg: "Failed to create path".to_string(),
                });
            }
        } else {
            copy_file(&path, blob, &bin.get_blobs_str()?, pulled_version.ver)?;
        }
    } else {
        info!("Content matches, not updating");
    }

    return Ok(());
}

/// Main handler for pulling configs from the Meld Bins
pub fn handler(main_args: Args, args: PullArgs) -> Result<(), libmeld::Error> {
    let bin = Bin::from(main_args.bin)?;

    let config_map_path = mapper::real_path_to_map(&args.config_path)?;

    // Look up the config in the db
    let blob = match bin.db.config_exists(&config_map_path)? {
        Some(b) => b,
        None => {
            return Err(Error::IOError {
                msg: "blob not found".to_string(),
            })
        }
    };

    info!("Config path matched: {}", blob);

    let map_blob = hash_path(&args.config_path);
    let map = bin.db.get_current_map(&map_blob)?;

    if map.is_none() {
        debug!("Config is single file; pull directly");
        pull_file(&bin, &blob, &args.tag, args.recent, args.version)?;
    } else {
        let map = map.unwrap();
        debug!("Config is map; parsing");
        let mut map_file_str: String = bin.get_maps_str()?;
        // build map file string from bin parts and found version
        map_file_str.push('/');
        map_file_str.push_str(&map_blob);
        map_file_str.push('-');
        map_file_str.push_str(&map.ver.to_string());
        info!("Parsing {}", map_file_str);
        let map_file = match fs::File::open(map_file_str) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::IOError { msg: e.to_string() });
            }
        };
        let map_reader = BufReader::new(map_file);

        for config in map_reader.lines() {
            match config {
                Ok(c) => {
                    let (blob, version) = c.split_once('-').unwrap();
                    debug!("Pulling {} V {}", blob, version);
                    pull_file(
                        &bin,
                        &blob.to_string(),
                        &args.tag,
                        args.recent,
                        version.parse::<u32>().unwrap(),
                    )?;
                }
                Err(e) => return Err(Error::IOError { msg: e.to_string() }),
            }
        }
    }

    return Ok(());
}
