use crate::Args;
use libmeld::{mapper, Bin, Config, Version};
use log::info;
use structopt::StructOpt;
use walkdir::WalkDir;

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

/// Push new config to Bin; we can infer things like version
/// TODO: figure out how to generate json map file
fn push_config(bin: &Bin, config: &Config) -> Result<(), libmeld::Error> {
    let cur_version = bin.db.get_current_version(config.get_blob())?;

    // if config is not in DB, add it
    // if config is in DB, determine updates
    if cur_version.is_none() {
        info!("Adding new config to bin");
        let version = Version {
            data_hash: Config::hash_contents(config.get_real_path())?,
            ver: 1,
            tag: config.get_tag().to_string(),
            owner: config.get_blob().to_string(),
        };

        bin.db.add_version(&version)?;
        bin.db.add_config(&config)?;

        return Ok(());
    }

    // handle versions table updates
    let cur_version = cur_version.unwrap();

    info!("Config exists in bin; determining needed updates");
    let new_hash = Config::hash_contents(config.get_real_path())?;
    let new_ver = cur_version.ver + 1;
    if cur_version.data_hash != new_hash {
        info!("Content differs; adding new version");
        let new_ver = Version {
            data_hash: new_hash,
            ver: new_ver,
            tag: config.get_tag().to_string(),
            owner: config.get_blob().to_string(),
        };
        bin.db.add_version(&new_ver)?;
    } else if cur_version.tag != config.get_tag().to_string() {
        info!("Tag differs; updating");
        bin.db.update_version_tag(&cur_version, config.get_tag())?;
    } else {
        info!("Config matches most recent version; no updates needed");
    }

    return Ok(());
}

/// Main handler for pushing configs to Meld Bins
pub fn handler(main_args: Args, args: PushArgs) -> Result<(), libmeld::Error> {
    let bin = Bin::from(main_args.bin)?;

    for entry in WalkDir::new(args.config_path) {
        match entry {
            Err(_) => (),
            Ok(d) => {
                if !d.path().is_dir() {
                    let str_path = d.path().to_str().unwrap().to_string();
                    let map_path = mapper::map_file(&str_path)?;
                    let config = Config::from(
                        str_path,
                        map_path,
                        args.subset.clone(),
                        args.family.clone(),
                        args.tag.clone(),
                    )?;
                    push_config(&bin, &config)?
                } else {
                    info!("Dir: {}", d.path().display());
                }
            }
        }
    }

    return Ok(());
}
