use crate::meld::MeldError;
use crate::Args;
use crate::{meld, util};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct InitArgs {
    // Create path if it doesn't exist
    #[structopt(
        short = "p",
        long = "parents",
        help = "make parent directories as needed"
    )]
    pub(crate) make_parents: bool,

    // Force use of folder
    #[structopt(short = "f", long = "force", help = "force use of an existing folder")]
    pub(crate) force: bool,
}

fn display_result(res: Result<(), MeldError>, debug: bool, pass_msg: &str) -> bool {
    match res {
        Ok(_) => {
            if debug {
                util::good_message(pass_msg);
            }
            return true;
        }
        Err(e) => {
            util::crit_message(&format!("Failed: {}", e));
            return false;
        }
    }
}

pub fn init_core(margs: Args, args: InitArgs) -> bool {
    let debug = margs.debug;
    let bin = meld::Bin::new(margs.bin);

    // Init Base Meld Bin
    let res = bin.create_bin_dir(args.make_parents, args.force, debug);
    if !display_result(res, debug, "Created Bin Dir") {
        return false;
    }

    // Create Meld Files in Meld Bin
    let res = bin.create_bin_files(args.force, debug);
    if !display_result(res, debug, "Created Bin Dir") {
        return false;
    }

    // Create Meld DB Schema
    let res = bin.create_db_schema(debug);
    if !display_result(res, debug, "Created DB Schema") {
        return false;
    }

    return true;
}

#[cfg(test)]
mod tests {
    use std::fs;

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
    fn new_bin() {
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
    fn create_parents() {
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
    fn force_bin() {
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
