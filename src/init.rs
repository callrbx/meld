use crate::util;
use crate::util::crit_message;
use crate::Args;
use rusqlite::{self, params, Connection};
use std::{fs, io::ErrorKind};
use structopt::StructOpt;

const INIT_SCHEMA: &str = "CREATE TABLE tracked (id TEXT, subset TEXT); CREATE TABLE versions (id TEXT, ver INTEGER, sphash TEXT)";

#[derive(Debug, StructOpt, Clone)]
pub struct InitArgs {
    // Create path if it doesn't exist
    #[structopt(
        short = "p",
        long = "parents",
        help = "make parent directories as needed"
    )]
    make_parents: bool,

    // Force use of folder
    #[structopt(short = "f", long = "force", help = "force use of an existing folder")]
    force: bool,
}

pub fn init_core(margs: Args, args: InitArgs) -> bool {
    let bin = margs.bin;
    let blobs_dir = format!("{}/blobs", &bin);
    let db = String::from(format!("{}/meld.db", &bin));

    if margs.debug {
        util::info_message(&format!("Intializing new bin {}", &bin));
    }

    if margs.debug && args.make_parents {
        util::info_message("Parent directories will be created");
    }

    if !args.force && util::path_exists(&bin) {
        util::crit_message("Directory exits and force not set");
        return false;
    }

    // initialize a new bin
    // create folder
    let dir_res = if !args.make_parents {
        fs::create_dir(&bin)
    } else {
        if margs.debug {
            util::info_message("Creating parent directories");
        }
        fs::create_dir_all(&bin)
    };
    match dir_res {
        Ok(_) => {
            if margs.debug {
                util::good_message(&format!("Created {}", &bin));
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                util::error_message(&format!("Folder {} exists", &bin));
                if args.force {
                    util::info_message(&format!("Forcing use of {}", &bin));
                } else {
                    util::crit_message("Not using existing folder");
                    return false;
                }
            }
            _ => {
                util::crit_message(&e.to_string());
                return false;
            }
        },
    }
    // Create blobs dir
    if fs::create_dir(blobs_dir).is_err() {
        crit_message("Failed to create blobs dir");
        return false;
    }

    // create sqlite db
    match fs::File::create(format!("{}", &db)) {
        Ok(_) => {
            if margs.debug {
                util::good_message(&format!("Created {}", &db));
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                util::error_message("Previous meld.db exists");
                if args.force {
                    util::info_message("Overwriting previous meld.db");
                } else {
                    util::crit_message("Not overwriting previous meld.db");
                    return false;
                }
            }
            _ => {
                util::crit_message(&e.to_string());
                return false;
            }
        },
    }

    // init sqlite db schema
    let con = match Connection::open(&db) {
        Ok(con) => {
            if margs.debug {
                util::info_message(&format!("Opened connection to {}", &db));
            }
            con
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    };
    match con.execute(INIT_SCHEMA, params![]) {
        Ok(con) => {
            if margs.debug {
                util::good_message("Successfully inited schema");
            }
        }
        Err(e) => {
            util::crit_message(&e.to_string());
            return false;
        }
    }

    return true;
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::Command;

    fn cleanup(path: &str) {
        match fs::remove_dir_all(path) {
            _ => {}
        }
    }

    // test the base case of a new bin in existing dir
    // no module arguments
    #[test]
    #[serial]
    fn test_base_case() {
        cleanup("/tmp/meld_test/");

        let margs = Args {
            debug: true,
            bin: String::from("/tmp/meld_test/"),
            command: Command::Init(InitArgs {
                make_parents: false,
                force: false,
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Init(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::init_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_test/"), true);
        assert_eq!(util::path_exists("/tmp/meld_test/meld.db"), true);

        cleanup("/tmp/meld_test/");
    }

    // test the base case of a new bin in a new dir
    // --parents
    #[test]
    #[serial]
    fn test_parents_case() {
        cleanup("/tmp/meld2/");

        // run without parent creation - should all fail
        let margs = Args {
            debug: true,
            bin: String::from("/tmp/meld2/meld_test/"),
            command: Command::Init(InitArgs {
                make_parents: false,
                force: false,
            }),
        };

        let mod_args = InitArgs {
            make_parents: false,
            force: false,
        };

        let res = super::init_core(margs, mod_args);
        assert_eq!(res, false);
        assert_eq!(util::path_exists("/tmp/meld2/meld_test/"), false);
        assert_eq!(util::path_exists("/tmp/meld2/meld_test/meld.db"), false);

        // run with parent creation option - should all pass
        let margs = Args {
            debug: true,
            bin: String::from("/tmp/meld2/meld_test/"),
            command: Command::Init(InitArgs {
                make_parents: true,
                force: false,
            }),
        };

        let mod_args = InitArgs {
            make_parents: true,
            force: false,
        };

        let res = super::init_core(margs, mod_args);
        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld2/meld_test/"), true);
        assert_eq!(util::path_exists("/tmp/meld2/meld_test/meld.db"), true);

        cleanup("/tmp/meld2/");
    }

    // test the case of reusing a dir
    // --force
    #[test]
    #[serial]
    fn test_force_case() {
        cleanup("/tmp/meld_test/");

        match fs::create_dir("/tmp/meld_test") {
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            _ => {}
        }

        // run without force - should all fail
        let margs = Args {
            debug: true,
            bin: String::from("/tmp/meld_test/"),
            command: Command::Init(InitArgs {
                make_parents: false,
                force: false,
            }),
        };

        let mod_args = InitArgs {
            make_parents: false,
            force: false,
        };

        let res = super::init_core(margs, mod_args);
        assert_eq!(res, false);
        assert_eq!(util::path_exists("/tmp/meld_test/"), true);

        // run with force option - should all pass
        let margs = Args {
            debug: true,
            bin: String::from("/tmp/meld_test/"),
            command: Command::Init(InitArgs {
                make_parents: false,
                force: true,
            }),
        };

        let mod_args = InitArgs {
            make_parents: false,
            force: true,
        };

        let res = super::init_core(margs, mod_args);
        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_test/"), true);
        assert_eq!(util::path_exists("/tmp/meld_test/meld.db"), true);

        cleanup("/tmp/meld_test/");
    }
}
