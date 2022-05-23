use crate::meld;
use crate::meld::Config;
use crate::meld::UpdateType;
use crate::util;
use crate::Args;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct PushArgs {
    #[structopt(short = "s", long = "subset", default_value = "", help = "subset tag")]
    pub(crate) subset: String,

    #[structopt(short = "t", long = "tag", default_value = "", help = "config tag")]
    pub(crate) tag: String,

    #[structopt(
        short = "f",
        long = "force",
        help = "force update (only effects dir configs)"
    )]
    pub(crate) force: bool,

    #[structopt(help = "config file/folder to add")]
    pub(crate) config_path: String,
}

fn push_file(config: &mut Config, debug: bool) -> bool {
    if debug {
        util::info_message(&format!("Attempting to track {}", config.blob_name))
    }

    let res = match config.get_update_type() {
        UpdateType::NewConfig => config.bin.add_config(&config),
        UpdateType::UpdateAll => config.bin.update_all(&config),
        UpdateType::UpdateSubset => config.bin.update_subset(&config),
        UpdateType::UpdateContent => config.bin.update_content(&config),
        UpdateType::NoUpdate => Ok(util::good_message("No Update Needed")),
    };

    match res {
        Ok(_) => {}
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    }

    if debug {
        util::info_message("Copying config to blobs");
    }

    return match config.bin.push_file(config) {
        Ok(_) => {
            if debug {
                util::good_message("Sucessfully copied config");
            }
            true
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    };
}

fn push_dir(config: &mut Config, debug: bool, force: bool) -> bool {
    if debug {
        util::info_message(&format!("Attempting to track {}", config.blob_name))
    }

    let res = match config.get_update_type() {
        UpdateType::NewConfig => config.bin.add_config(&config),
        UpdateType::UpdateSubset => config.bin.update_subset(&config),
        UpdateType::NoUpdate => Ok(util::good_message("No Update Needed")),
        UpdateType::UpdateContent | UpdateType::UpdateAll => {
            config.bin.update_subset(&config).unwrap();
            if !force {
                util::crit_message("Dir config exist; use -f to overwrite");
                std::process::exit(1);
            } else {
                util::info_message("Updating all files in config dir");
                config.bin.update_content(config)
            }
        }
    };

    match res {
        Ok(_) => {}
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    }

    if debug {
        util::info_message("Copying config dir to blobs");
    }

    return match config.bin.push_dir(config) {
        Ok(_) => {
            if debug {
                util::good_message("Sucessfully copied config");
            }
            true
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    };
}

pub fn push_core(margs: Args, args: PushArgs) -> bool {
    let bin = meld::Bin::new(margs.bin);

    let mut config = match meld::Config::new(args.config_path, args.subset, args.tag, bin, false) {
        Err(e) => {
            util::crit_message(&format!("{}", e));
            return false;
        }
        Ok(c) => c,
    };

    if margs.debug {
        util::info_message(&format!("Pushing config {}", config.real_path));
    }

    return if !config.is_dir {
        push_file(&mut config, margs.debug)
    } else {
        push_dir(&mut config, margs.debug, args.force)
    };
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use serial_test::serial;

    use super::*;
    use crate::{
        init::{init_core, InitArgs},
        Command,
    };

    const TEST_BIN: &str = "/tmp/meld_push_test/";
    const TEST_CONF: &str = "/tmp/meld_push_test.config";

    fn cleanup() {
        match fs::remove_dir_all(TEST_BIN) {
            _ => {}
        }

        match fs::remove_file(TEST_CONF) {
            _ => {}
        }
    }

    // init a new bin and create a sample file
    fn init_bin() {
        let mut file = fs::File::create(TEST_CONF).unwrap();
        file.write_all("test=true\n".as_bytes()).unwrap();

        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Init(InitArgs {
                make_parents: false,
                force: false,
            }),
        };

        let mod_args = InitArgs {
            make_parents: false,
            force: false,
        };

        init_core(margs, mod_args);
    }

    // test adding that sample file to the bin
    #[test]
    #[serial]
    fn push_config() {
        cleanup();
        init_bin();

        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Push(PushArgs {
                config_path: TEST_CONF.to_string(),
                subset: "".to_string(),
                tag: "".to_string(),
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Push(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::push_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/"), true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/1"), true);
    }

    // modify that file and test the version is updated
    #[test]
    #[serial]
    fn push_update_config() {
        cleanup();
        init_bin();

        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Push(PushArgs {
                config_path: TEST_CONF.to_string(),
                subset: "".to_string(),
                tag: "".to_string(),
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Push(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::push_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/"), true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/1"), true);

        let mut file = fs::File::create(TEST_CONF).unwrap();
        file.write_all("test=false\n".as_bytes()).unwrap();

        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Push(PushArgs {
                config_path: TEST_CONF.to_string(),
                subset: "".to_string(),
                tag: "".to_string(),
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Push(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::push_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/"), true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/1"), true);
        assert_eq!(util::path_exists("/tmp/meld_push_test/blobs/e37329b0255f680a3384bc0161182d7448097fc0e5a9a5827437b873f600b5a5790fa7d619323e4212318f406cda6644c2eb60a20030dc48264678ca3137b767/2"), true);

        cleanup();
    }
}
