use crate::util;
use rusqlite::{params, Connection};
use snafu::{self, Snafu};
use std::{collections::HashMap, fs};

const INIT_TRACKED: &str = "CREATE TABLE tracked (id TEXT, subset TEXT)";
const INIT_VERSIONS: &str = "CREATE TABLE versions (id TEXT, ver INTEGER, sphash TEXT)";

#[derive(Debug, Snafu)]
pub enum MeldError {
    #[snafu(display("Bin Already Exists"))]
    BinAlreadyExists,
    #[snafu(display("Dir Creation Failed '{error_msg}'"))]
    DirCreateFailed { error_msg: String },
    #[snafu(display("File Creation Failed '{error_msg}'"))]
    FileCreateFailed { error_msg: String },
    #[snafu(display("File Already Exists"))]
    FileAlreadyExists,
    #[snafu(display("Meld SQL Failed '{error_msg}'"))]
    MeldSqlFailed { error_msg: String },
}

pub struct Bin {
    pub path: String,
    pub blobs: String,
    pub db: String,
}

impl Bin {
    pub fn new(path: String) -> Self {
        Bin {
            path: path.clone(),
            blobs: format!("{}/{}", path.clone(), "blobs"),
            db: format!("{}/{}", path.clone(), "meld.db"),
        }
    }

    pub fn create_bin_dir(&self, parents: bool, force: bool, debug: bool) -> Result<(), MeldError> {
        if debug {
            util::info_message(&format!("Intializing new bin {}", self.path));
            if parents {
                util::info_message("Parent directories will be created");
            }
        }

        let bin_exists = util::path_exists(&self.path);

        // If bin directory exists and force is not set
        if bin_exists && !force {
            return BinAlreadyExistsSnafu.fail();
        }

        // Create dir or create dir with parents
        if !bin_exists {
            let bin_create_res = if parents {
                fs::create_dir_all(&self.path)
            } else {
                fs::create_dir(&self.path)
            };

            // Evaluate Dir Creation for success
            match bin_create_res {
                Err(e) => {
                    return DirCreateFailedSnafu {
                        error_msg: e.to_string(),
                    }
                    .fail();
                }
                Ok(_) => {}
            }
        }

        return Ok(());
    }

    pub fn create_bin_files(&self, force: bool, debug: bool) -> Result<(), MeldError> {
        if util::path_exists(&self.blobs) {
            if force {
                if debug {
                    util::info_message("Removing existing blobs dir");
                }
                fs::remove_dir_all(&self.blobs).unwrap();
            } else {
                return FileAlreadyExistsSnafu.fail();
            }
        }

        match fs::create_dir(&self.blobs) {
            Err(e) => {
                return DirCreateFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail();
            }
            Ok(_) => {}
        }

        if util::path_exists(&self.db) {
            if force {
                if debug {
                    util::info_message("Removing existing db file");
                }
                fs::remove_file(&self.db).unwrap();
            } else {
                return FileAlreadyExistsSnafu.fail();
            }
        }

        match fs::File::create(&self.db) {
            Err(e) => {
                return FileCreateFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail();
            }
            Ok(_) => {}
        }

        return Ok(());
    }

    pub fn create_db_schema(&self, debug: bool) -> Result<(), MeldError> {
        let con = match Connection::open(&self.db) {
            Ok(con) => {
                if debug {
                    util::info_message("Opened connection to db");
                }
                con
            }
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        };
        match con.execute(INIT_TRACKED, params![]) {
            Ok(_) => {
                if debug {
                    util::good_message("Successfully inited tracked schema");
                }
            }
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }

        match con.execute(INIT_VERSIONS, params![]) {
            Ok(_) => {
                if debug {
                    util::good_message("Successfully inited versions schema");
                }
            }
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }

        return Ok(());
    }
}

pub struct Config {
    is_dir: bool,
    real_path: String,
    store_path: String,
    versions: HashMap<i32, String>,
}
