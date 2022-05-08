use crate::util;
use crate::Args;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct PushArgs {
    #[structopt(
        short = "s",
        long = "subset",
        default_value = "",
        help = "config file/folder to add"
    )]
    subset: String,

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

fn push_file(bin: String, db: String, path: String, subset: String, debug: bool) -> io::Result<()> {
    let blobs_dir = format!("{}/blobs", bin);

    if debug {
        util::info_message(&format!("Attempting to track {}", path))
    }

    // TODO path translations
    let abs_path = fs::canonicalize(path).unwrap();
    let store_path = abs_path.to_str().unwrap();

    if debug {
        util::info_message(&format!("Tracking as {}", store_path))
    }

    let blob_name = util::hash_path(&store_path);
    let blob_content_hash = util::hash_contents(&abs_path.to_str().unwrap());

    if debug {
        util::info_message(&format!("Blob Name: {}/{}", blobs_dir, blob_name))
    }

    let config_blob_dir = format!("{}/{}", blobs_dir, blob_name);

    // check if config is already tracked
    let is_update = config_exists(&db, &blob_name);
    if debug && is_update {
        util::info_message("Updating existing config");
    }

    // set version and create new blob dir if needed
    let version = if is_update {
        util::crit_message("update not currently supported");
        std::process::exit(1);
    } else {
        // create configs blob folder
        fs::create_dir(config_blob_dir)?;
        "1"
    };

    // track config in meld.db
    let con = match Connection::open(db) {
        Ok(con) => con,
        Err(e) => {
            util::crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    if !is_update {
        con.execute(
            "INSERT INTO tracked (id, subset) VALUES (?1, ?2)",
            params![blob_name, subset],
        )
        .unwrap();
        con.execute(
            "INSERT INTO versions (id, ver, sphash) VALUES (?1, ?2, ?3)",
            params![blob_content_hash, version, blob_name],
        )
        .unwrap();
    }

    // copy config blob to proper directory
    let dest_path = format!("{}/{}/{}", blobs_dir, blob_name, version);
    fs::copy(abs_path, dest_path)?;

    return Ok(());
}

pub fn push_core(margs: Args, args: PushArgs) -> bool {
    let bin = margs.bin;
    let db = String::from(format!("{}/meld.db", &bin));

    // check meld bin is configured properly
    if !util::valid_meld_dir(&bin) {
        util::crit_message(&format!("{} is not a valid meld bin", bin));
        return false;
    } else if margs.debug {
        util::info_message(&format!("Using bin {}", bin));
    }

    // check config_path exists
    if !util::path_exists(&args.config_path) {
        util::crit_message(&format!("{} does not exist", args.config_path));
        return false;
    } else if margs.debug {
        util::info_message(&format!("Using {}", args.config_path));
    }

    // determine if config is dir or folder
    if util::is_dir(&args.config_path) {
        util::crit_message("currently unsupported");
        return false;
    } else {
        if push_file(bin, db, args.config_path, args.subset, margs.debug).is_err() {
            return false;
        }
    }

    return true;
}
