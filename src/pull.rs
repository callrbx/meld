use crate::util;
use crate::Args;
use rusqlite::params;
use rusqlite::Connection;

use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(help = "config file/folder to add")]
    config_path: String,
}

fn config_exists(db: &str, blob_name: &str) -> bool {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            util::crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con.prepare("SELECT * FROM tracked WHERE id = ?").unwrap();

    return match stmt.exists(params![blob_name]) {
        Ok(b) => b,
        Err(e) => {
            util::error_message(&e.to_string());
            return false;
        }
    };
}

pub fn pull_core(margs: Args, args: PullArgs) -> bool {
    let bin = margs.bin;
    let db = String::from(format!("{}/meld.db", &bin));

    // check meld bin is configured properly
    if !util::valid_meld_dir(&bin) {
        util::crit_message(&format!("{} is not a valid meld bin", bin));
        return false;
    } else if margs.debug {
        util::info_message(&format!("Using bin {}", bin));
    }

    return true;
}
