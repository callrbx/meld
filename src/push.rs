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

fn get_next_version(db: &str, blob_name: &str) -> i32 {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            util::crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con
        .prepare("SELECT ver FROM versions WHERE sphash = ? ORDER BY ver DESC")
        .unwrap();
    let mut rows = stmt.query(params![blob_name]).unwrap();

    let last_ver: i32 = rows.next().unwrap().unwrap().get(0).unwrap();

    return last_ver + 1;
}

fn is_update_needed(db: &str, blob_name: &str, content_hash: &str) -> bool {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            util::crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con
        .prepare("SELECT id FROM versions WHERE sphash = ? ORDER BY ver DESC")
        .unwrap();
    let mut rows = stmt.query(params![blob_name]).unwrap();

    let stored_content_hash: String = rows.next().unwrap().unwrap().get(0).unwrap();

    // no updated needed if hashes match
    return !(stored_content_hash == content_hash);
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

    if is_update {
        if debug {
            util::info_message("Checking if update is needed");
        }
        if !is_update_needed(&db, &blob_name, &blob_content_hash) {
            util::good_message("No update is needed");
            return Ok(());
        } else if debug {
            util::info_message("Content Changed, updating")
        }
    }

    // set version and create new blob dir if needed
    let version = if is_update {
        get_next_version(&db, &blob_name)
    } else {
        // create configs blob folder
        fs::create_dir(config_blob_dir)?;
        1
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serial_test::serial;

    use super::*;
    use crate::{
        init::{init_core, InitArgs},
        Command,
    };

    const TEST_BIN: &str = "/tmp/meld_test/";
    const TEST_CONF: &str = "/tmp/meld_test.config";

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
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Push(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::push_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_test/blobs/60bf786278486a27e903dd86aafa958e712ca6896fada19e907a443006198048a4a046f25f87d09d25ee2c16b1f3fc4834ba93dc991e054922018d1f97bf3ee7/"), true);
        assert_eq!(util::path_exists("/tmp/meld_test/blobs/60bf786278486a27e903dd86aafa958e712ca6896fada19e907a443006198048a4a046f25f87d09d25ee2c16b1f3fc4834ba93dc991e054922018d1f97bf3ee7/1"), true);
    }

    // modify that file and test the version is updated
    #[test]
    #[serial]
    fn push_update_config() {
        let mut file = fs::File::create(TEST_CONF).unwrap();
        file.write_all("test=false\n".as_bytes()).unwrap();

        let margs = Args {
            debug: true,
            bin: String::from(TEST_BIN),
            command: Command::Push(PushArgs {
                config_path: TEST_CONF.to_string(),
                subset: "".to_string(),
            }),
        };

        let mod_args = match margs.clone().command {
            Command::Push(a) => a,
            _ => std::process::exit(1),
        };

        let res = super::push_core(margs, mod_args);

        assert_eq!(res, true);
        assert_eq!(util::path_exists("/tmp/meld_test/blobs/60bf786278486a27e903dd86aafa958e712ca6896fada19e907a443006198048a4a046f25f87d09d25ee2c16b1f3fc4834ba93dc991e054922018d1f97bf3ee7/"), true);
        assert_eq!(util::path_exists("/tmp/meld_test/blobs/60bf786278486a27e903dd86aafa958e712ca6896fada19e907a443006198048a4a046f25f87d09d25ee2c16b1f3fc4834ba93dc991e054922018d1f97bf3ee7/1"), true);
        assert_eq!(util::path_exists("/tmp/meld_test/blobs/60bf786278486a27e903dd86aafa958e712ca6896fada19e907a443006198048a4a046f25f87d09d25ee2c16b1f3fc4834ba93dc991e054922018d1f97bf3ee7/2"), true);

        cleanup();
    }
}
