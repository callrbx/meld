use crate::meld;
use crate::meld::Config;
use crate::util;
use crate::Args;

use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(short = "v", long = "version", help = "revert to a specific version")]
    version: Option<String>,

    #[structopt(short = "f", long = "force", help = "force update on existing dir")]
    force: bool,

    #[structopt(help = "config object to pull")]
    config: String,
}

fn pull_file(config: &mut Config, debug: bool, version: Option<String>) -> bool {
    if debug {
        util::info_message(&format!("Pulling config file {}", config.blob_name));
    }

    return match config.bin.pull_file(config, version) {
        Ok(_) => {
            if debug {
                util::good_message("Config pulled successfully");
            }
            true
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            false
        }
    };
}

fn pull_dir(config: &mut Config, debug: bool, version: Option<String>, force: bool) -> bool {
    if debug {
        util::info_message(&format!("Pulling config dir {}", config.blob_name));
    }

    // Match last tracked version or user specified
    return match config.bin.pull_dir(config, version, force) {
        Ok(_) => {
            if debug {
                util::good_message("Config pulled successfully");
            }
            true
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            false
        }
    };
}

pub fn pull_core(margs: Args, args: PullArgs) -> bool {
    let bin = meld::Bin::new(margs.bin);

    let mut config = match meld::Config::new(args.config, "".to_string(), "".to_string(), bin, true)
    {
        Err(e) => {
            util::crit_message(&format!("{}", e));
            return false;
        }
        Ok(c) => c,
    };

    if margs.debug {
        util::info_message(&format!("Pulling config {}", config.real_path));
    }

    return if !config.is_dir {
        pull_file(&mut config, margs.debug, args.version)
    } else {
        pull_dir(&mut config, margs.debug, args.version, args.force)
    };
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use serial_test::serial;

    use super::*;
    use crate::{
        init::{init_core, InitArgs},
        push::PushArgs,
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

    // add a sample config to the bin
    fn push_config() {
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

        crate::push::push_core(margs, mod_args);
    }

    // test pulling a new config file
    #[test]
    #[serial]
    fn pull_new_config() {
        // setup code - init bin, add file, remove file for "new"
        cleanup();
        init_bin();
        push_config();
        fs::remove_file(TEST_CONF).unwrap();

        // test code
        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Pull(PullArgs {
                config: TEST_CONF.to_string(),
                version: None,
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Pull(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::pull_core(margs, mod_args);
        assert_eq!(res, true);
        assert_eq!(util::path_exists(TEST_CONF), true);
    }

    // test pulling a good reversion
    #[test]
    #[serial]
    fn pull_good_reversion() {
        // setup code - init bin, add file, remove file for "new"
        cleanup();
        init_bin();
        push_config();
        fs::remove_file(TEST_CONF).unwrap();

        // test code
        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Pull(PullArgs {
                config: TEST_CONF.to_string(),
                version: Some("1".to_string()),
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Pull(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::pull_core(margs, mod_args);
        assert_eq!(res, true);
        assert_eq!(util::path_exists(TEST_CONF), true);
    }

    // test pulling a bad reversion
    #[test]
    #[serial]
    fn pull_bad_reversion() {
        // setup code - init bin, add file, remove file for "new"
        cleanup();
        init_bin();
        push_config();
        fs::remove_file(TEST_CONF).unwrap();

        // test code
        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Pull(PullArgs {
                config: TEST_CONF.to_string(),
                version: Some("100".to_string()), // bad reversion
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Pull(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::pull_core(margs, mod_args);
        assert_eq!(res, false);
        assert_eq!(util::path_exists(TEST_CONF), false);
    }
}
