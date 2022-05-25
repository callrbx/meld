use std::{
    fs::{self, File},
    io::Write,
};

use crate::Args;
use libmeld::{hash_contents, is_dir, mapper, Bin, Config, Error, Map, Version};
use log::{debug, info};
use structopt::StructOpt;

// Define Module Arguments
#[derive(Debug, StructOpt, Clone)]
pub struct PushArgs {
    #[structopt(
        short = "s",
        long = "subset",
        default_value = "",
        help = "config subset"
    )]
    pub(crate) subset: String,

    #[structopt(
        short = "f",
        long = "family",
        default_value = "",
        help = "config family"
    )]
    pub(crate) family: String,

    #[structopt(short = "t", long = "tag", default_value = "", help = "config tag")]
    pub(crate) tag: String,

    #[structopt(help = "config file/folder to add")]
    pub(crate) config_path: String,
}

/// Function to actually copy configs to the blobs directory
fn copy_file(
    path: &String,
    blob_name: &String,
    blobs_dir: &String,
    version: u32,
) -> Result<(), Error> {
    // ignore dirs if the are copied
    if is_dir(&path)? {
        return Ok(());
    }

    // construct blob and version path for config
    let blob_ver_path = format!("{}/{}/{}", blobs_dir, blob_name, version);

    debug!("copy {} -> {}", path, blob_ver_path);

    return match fs::copy(path, blob_ver_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::IOError { msg: e.to_string() }),
    };
}

/// Push new config to Bin
/// will determine updates needed per config
/// Returns the version of either currently tracked config
fn push_config(bin: &Bin, config: &Config) -> Result<u32, libmeld::Error> {
    let cur_version = bin.db.get_current_version(config.get_blob())?;

    // if config is not in DB, add it
    // if config is in DB, determine updates
    if cur_version.is_none() {
        info!("Adding new config to bin");
        let version = Version {
            data_hash: hash_contents(config.get_real_path())?,
            ver: 1,
            tag: config.get_tag().to_string(),
            owner: config.get_blob().to_string(),
        };

        // create the blob dir
        match fs::create_dir(format!("{}/{}", bin.get_blobs_str()?, config.get_blob())) {
            Ok(_) => (),
            Err(e) => return Err(Error::IOError { msg: e.to_string() }),
        }

        // copy to blobs
        copy_file(
            config.get_real_path(),
            config.get_blob(),
            &bin.get_blobs_str()?,
            1,
        )?;

        // add to db after a successful copy
        bin.db.add_version(&version)?;
        bin.db.add_config(&config)?;

        // version is one since just added
        return Ok(1);
    }

    // handle versions table updates
    let cur_version = cur_version.unwrap();

    info!("Config exists in bin; determining needed updates");
    let new_hash = hash_contents(config.get_real_path())?;
    let new_ver = cur_version.ver + 1;

    // do proper update action; return the current version num in db
    let db_ver = if cur_version.data_hash != new_hash {
        info!("Content differs; adding new version");
        let new_ver = Version {
            data_hash: new_hash,
            ver: new_ver,
            tag: config.get_tag().to_string(),
            owner: config.get_blob().to_string(),
        };

        // copy to blobs
        copy_file(
            config.get_real_path(),
            config.get_blob(),
            &bin.get_blobs_str()?,
            1,
        )?;

        // add to db after good copy
        bin.db.add_version(&new_ver)?;

        new_ver.ver
    } else if cur_version.tag != config.get_tag().to_string() {
        info!("Tag differs; updating");
        bin.db.update_version_tag(&cur_version, config.get_tag())?;
        cur_version.ver
    } else {
        info!("Config matches most recent version; no updates needed");
        cur_version.ver
    };

    // return the new version
    return Ok(db_ver);
}

/// Main handler for pushing configs to Meld Bins
pub fn handler(main_args: Args, args: PushArgs) -> Result<(), libmeld::Error> {
    let bin = Bin::from(main_args.bin)?;

    // handle single file config pushes
    if !is_dir(&args.config_path)? {
        info!("Pushing single file");
        let map_path = mapper::map_file(&args.config_path)?;
        let config = Config::from(
            args.config_path,
            map_path,
            args.subset.clone(),
            args.family.clone(),
            args.tag.clone(),
        )?;
        push_config(&bin, &config)?;
    } else {
        info!("Pushing dir tree");
        // create map and add to db
        let mut map = Map::new(&args.config_path, args.subset, args.family, args.tag)?;
        info!("Map contains {} configs", map.configs.len());

        // check if map exists; if it does, check if the hashes match
        // update the map version accordingly; 0 if update not needed
        map.ver = match bin.db.get_current_map(&map.blob)? {
            Some(m) => {
                info!(
                    "Map for {} exists; determining if update needed",
                    args.config_path
                );
                if m.hash == map.hash {
                    info!("Stored map matches current map; not updating");
                    0
                } else {
                    m.ver + 1
                }
            }
            None => {
                info!("Map not in db; adding");
                1
            }
        };
        // add the map to the db if new or not matching most recent hash
        if map.ver != 0 {
            bin.db.add_map(&map)?;
            let mut map_file =
                match File::create(format!("{}/{}-{}", bin.get_maps_str()?, map.blob, map.ver)) {
                    Ok(f) => f,
                    Err(e) => return Err(Error::IOError { msg: e.to_string() }),
                };

            // push all the configs in the map and write map file
            // each config will update and track state separetly
            for c in map.configs {
                info!("Handling Config {}", c.get_real_path());
                let cv = push_config(&bin, &c)?;
                match map_file.write_fmt(format_args!("{}-{}\n", c.get_blob(), cv)) {
                    Ok(_) => (),
                    Err(e) => return Err(Error::IOError { msg: e.to_string() }),
                };
            }
        } else {
            // if we dont need to rewrite the config
            // still update the configs as contents may have changed
            for c in map.configs {
                push_config(&bin, &c)?;
            }
        }
    }

    return Ok(());
}
