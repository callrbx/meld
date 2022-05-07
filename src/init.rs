use sqlite;
use std::fs;

use crate::MainArgs;
use structopt::StructOpt;

use crate::util;

#[derive(StructOpt)]
#[structopt(name = "meld init subcommand", author = "drew <drew@parker.systems>")]
struct InitArgs {
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

pub fn init_core(margs: MainArgs, args: Vec<String>) -> bool {
    let args = InitArgs::from_iter(args);

    let bin = margs.bin;
    let db = String::from(format!("{}/meld.db", &bin));

    if margs.debug {
        util::info_message(&format!("Intializing new bin {}", &bin));
    }

    if margs.debug && args.make_parents {
        util::info_message("Parent directories will be created");
    }

    // ensure we dont existing folders without good reason
    if util::path_exists(&bin) {
        if !args.force {
            util::crit_message(&format!("Folder {} exists", &bin));
            return false;
        } else {
            util::info_message(&format!("Forcing use of {}", &bin));
        }
    }

    // initialize a new bin
    // create folder
    match fs::create_dir(&bin) {
        Ok(_) => {
            if margs.debug {
                util::good_message(&format!("Created {}", &bin));
            }
        }
        Err(e) => util::crit_message(&e.to_string()),
    }

    // create sqlite db
    match fs::File::create(format!("{}", &db)) {
        Ok(_) => {
            if margs.debug {
                util::good_message(&format!("Created {}", &db));
            }
        }
        Err(e) => util::crit_message(&e.to_string()),
    }

    // init sqlite db schema
    let con = sqlite::open(db);

    return true;
}
