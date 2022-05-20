use crate::util;
use crate::Args;
use path_abs;
use path_abs::PathInfo;
use std::fs;

use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(short = "r", long = "revert", help = "revert to a specific version")]
    revert: Option<String>,

    #[structopt(help = "config object to pull")]
    config: String,
}

fn copy_config_dir() -> bool {
    //     if debug {
    //         util::info_message(&format!("Pulling config dir {}", blob_name));
    //     }
    util::crit_message("currently unsupported");

    return false;
}

fn copy_config_file(
    bin: &str,
    db: &str,
    real_path: &str,
    blob_name: &str,
    debug: bool,
    revert: Option<String>,
) -> bool {
    if debug {
        util::info_message(&format!("Pulling config file {}", blob_name));
    }

    let vers = match revert {
        Some(r) => r,
        None => util::get_cur_version(&db, &blob_name).to_string(),
    };
    let blob_ver_path = format!("{}/blobs/{}/{}", bin, blob_name, vers);
    if !util::path_exists(&blob_ver_path) {
        util::crit_message(&format!("Could not find {}", blob_ver_path));
        return false;
    }

    return match fs::copy(blob_ver_path, real_path) {
        Ok(_) => true,
        Err(e) => {
            util::crit_message(&format!("Failed to copy: {}", e));
            false
        }
    };
}

pub fn pull_core(margs: Args, args: PullArgs) -> bool {
    return true;
}

#[cfg(test)]
mod tests {
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
                revert: None,
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
                revert: Some("1".to_string()),
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
                revert: Some("100".to_string()), // bad reversion
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
