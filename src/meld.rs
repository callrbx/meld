use crate::util::{self};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha512};
use snafu::{self, Snafu};
use std::{collections::HashMap, fs};

const INIT_TRACKED: &str = "CREATE TABLE tracked (id TEXT, path TEXT, subset TEXT)";
const INIT_VERSIONS: &str = "CREATE TABLE versions (id TEXT, ver INTEGER, tag TEXT, sphash TEXT)";

#[derive(Debug, Snafu)]
pub enum MeldError {
    #[snafu(display("Bin Already Exists"))]
    BinAlreadyExists,
    #[snafu(display("Dir Already Exists"))]
    DirAlreadyExists,
    #[snafu(display("Dir Creation Failed '{error_msg}'"))]
    DirCreateFailed { error_msg: String },
    #[snafu(display("File Creation Failed '{error_msg}'"))]
    FileCreateFailed { error_msg: String },
    #[snafu(display("File Already Exists"))]
    FileAlreadyExists,
    #[snafu(display("Meld SQL Failed '{error_msg}'"))]
    MeldSqlFailed { error_msg: String },
    #[snafu(display("Invalid Meld Bin"))]
    InvalidBin,
    #[snafu(display("Invalid Config File"))]
    InvalidConfig,
    #[snafu(display("Failed to create Blob Dir '{error_msg}'"))]
    MeldBlobDirFailed { error_msg: String },
    #[snafu(display("Failed to copy Config File '{error_msg}'"))]
    MeldFileCopyFailed { error_msg: String },
    #[snafu(display("Failed to copy Config Dir '{error_msg}'"))]
    MeldDirCopyFailed { error_msg: String },
    #[snafu(display("Config dir already exists"))]
    ConfigDirExists,
    #[snafu(display("Meld Blob Not Found"))]
    BlobNotFound,
}

pub enum UpdateType {
    NewConfig,
    UpdateSubset,
    UpdateContent,
    UpdateAll,
    NoUpdate,
}

pub struct Bin {
    pub path: String,
    pub blobs: String,
    pub db: String,
}

impl Bin {
    pub fn is_valid(&self) -> bool {
        return util::path_exists(&self.path)
            && util::path_exists(&self.blobs)
            && util::path_exists(&self.db);
    }

    pub fn get_cur_version(&self, config: &Config) -> i32 {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        let mut stmt = con
            .prepare("SELECT ver FROM versions WHERE sphash = ? ORDER BY ver DESC")
            .unwrap();
        let mut rows = stmt.query(params![config.blob_name]).unwrap();

        let last_ver: i32 = rows.next().unwrap().unwrap().get(0).unwrap();

        return last_ver;
    }

    pub fn config_exists(&self, blob_name: &str) -> bool {
        let con = match Connection::open(&self.db) {
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

    pub fn config_is_dir(&self, config: &Config) -> bool {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        let mut stmt = con
            .prepare("SELECT id FROM versions WHERE sphash = ? ORDER BY ver DESC")
            .unwrap();
        let mut rows = stmt.query(params![config.blob_name]).unwrap();

        let id: String = rows.next().unwrap().unwrap().get(0).unwrap();

        return id == "DIR";
    }

    pub fn is_update_content_needed(&self, config: &Config) -> bool {
        if config.is_dir {
            return false;
        }

        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        let mut stmt = con
            .prepare("SELECT id FROM versions WHERE sphash = ? ORDER BY ver DESC")
            .unwrap();
        let mut rows = stmt.query(params![config.blob_name]).unwrap();

        let stored_content_hash: String = rows.next().unwrap().unwrap().get(0).unwrap();

        // no updated needed if hashes match
        return !(stored_content_hash == config.content_hash);
    }

    pub fn update_content(&self, config: &Config) -> Result<(), MeldError> {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        match con.execute(
            "INSERT INTO versions (id, ver, tag, sphash) VALUES (?1, ?2, ?3, ?4)",
            params![
                config.content_hash,
                config.version,
                config.tag,
                config.blob_name
            ],
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }
    }

    pub fn is_update_subset_needed(&self, config: &Config) -> bool {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        let mut stmt = con
            .prepare("SELECT subset FROM tracked WHERE id = ?")
            .unwrap();
        let mut rows = stmt.query(params![config.blob_name]).unwrap();

        let stored_subset: String = rows.next().unwrap().unwrap().get(0).unwrap();

        // no updated needed if hashes match
        return !(stored_subset == config.subset) || !(config.subset == "");
    }

    pub fn update_subset(&self, config: &Config) -> Result<(), MeldError> {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                util::crit_message(&e.to_string());
                std::process::exit(1);
            }
        };

        match con.execute(
            "UPDATE tracked SET subset=?2 WHERE id=?1",
            params![config.blob_name, config.subset],
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }
    }

    // Update Subset and Content
    pub fn update_all(&self, config: &Config) -> Result<(), MeldError> {
        if Bin::update_subset(&self, config).is_err() {
            return MeldSqlFailedSnafu {
                error_msg: "Failed to update subset",
            }
            .fail();
        }

        if Bin::update_content(&self, config).is_err() {
            return MeldSqlFailedSnafu {
                error_msg: "Failed to update subset",
            }
            .fail();
        }

        return Ok(());
    }

    // Add a new config to tracked and versions
    pub fn add_config(&self, config: &Config) -> Result<(), MeldError> {
        let con = match Connection::open(&self.db) {
            Ok(con) => con,
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        };

        match con.execute(
            "INSERT INTO tracked (id, path, subset) VALUES (?1, ?2, ?3)",
            params![config.blob_name, config.real_path, config.subset],
        ) {
            Ok(_) => {}
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }

        match con.execute(
            "INSERT INTO versions (id, ver, tag, sphash) VALUES (?1, ?2, ?3, ?4)",
            params![config.content_hash, 1, config.tag, config.blob_name],
        ) {
            Ok(_) => {}
            Err(e) => {
                return MeldSqlFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        }

        return Ok(());
    }

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

    pub fn push_file(&self, config: &Config) -> Result<(), MeldError> {
        let blob_dir = format!("{}/{}", self.blobs, config.blob_name);
        let dest_path = format!("{}/{}/{}", self.blobs, config.blob_name, config.version);

        if !util::path_exists(&blob_dir) {
            match fs::create_dir(blob_dir) {
                Ok(_) => {}
                Err(e) => {
                    return MeldBlobDirFailedSnafu {
                        error_msg: e.to_string(),
                    }
                    .fail()
                }
            };
        }

        return match fs::copy(&config.real_path, dest_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                return MeldFileCopyFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        };
    }

    pub fn push_dir(&self, config: &Config) -> Result<(), MeldError> {
        let dest_path = format!("{}/{}/{}", self.blobs, config.blob_name, config.version);

        if !util::path_exists(&dest_path) {
            match fs::create_dir_all(&dest_path) {
                Ok(_) => {}
                Err(e) => {
                    return MeldBlobDirFailedSnafu {
                        error_msg: e.to_string(),
                    }
                    .fail()
                }
            };
        }

        let mut opt = fs_extra::dir::CopyOptions::new();
        opt.content_only = true;
        return match fs_extra::dir::copy(&config.real_path, &dest_path, &opt) {
            Ok(_) => Ok(()),
            Err(e) => {
                return MeldDirCopyFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail()
            }
        };
    }

    pub fn pull_file(&self, config: &Config, version: Option<String>) -> Result<(), MeldError> {
        // Match last tracked version or user specified
        let vers = match version {
            Some(r) => r,
            None => config.bin.get_cur_version(&config).to_string(),
        };

        // Build Blob path w/ version
        let blob_ver_path = match config.get_blob_version_path(vers) {
            Ok(path) => path,
            Err(e) => {
                util::crit_message(&e.to_string());
                return BlobNotFoundSnafu.fail();
            }
        };

        match fs::copy(blob_ver_path, &config.real_path) {
            Ok(_) => true,
            Err(e) => {
                util::crit_message(&format!("Failed to copy: {}", e));
                return MeldFileCopyFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail();
            }
        };

        return Ok(());
    }

    pub fn pull_dir(
        &self,
        config: &Config,
        version: Option<String>,
        force: bool,
    ) -> Result<(), MeldError> {
        // Match last tracked version or user specified
        let vers = match version {
            Some(r) => r,
            None => config.bin.get_cur_version(&config).to_string(),
        };

        // Build Blob path w/ version
        let blob_ver_path = match config.get_blob_version_path(vers) {
            Ok(path) => path,
            Err(e) => {
                util::crit_message(&e.to_string());
                return BlobNotFoundSnafu.fail();
            }
        };

        let path_exists = util::path_exists(&config.real_path);

        // check if path exists; if yes and not force, fail
        if path_exists && !force {
            util::crit_message("Directory already exists and force not set");
            return DirAlreadyExistsSnafu.fail();
        } else if path_exists && force {
            util::info_message("Forcing updating of dir; removing existing dir");
            match fs::remove_dir(&config.real_path) {
                Err(e) => {
                    util::crit_message("Failed to remove existing dir");
                    return MeldDirCopyFailedSnafu {
                        error_msg: e.to_string(),
                    }
                    .fail();
                }
                _ => {}
            }
        } else if !path_exists {
            match fs::create_dir(&config.real_path) {
                Err(e) => {
                    util::crit_message("Failed to create dir");
                    return MeldDirCopyFailedSnafu {
                        error_msg: e.to_string(),
                    }
                    .fail();
                }
                _ => {}
            }
        }

        let mut opt = fs_extra::dir::CopyOptions::new();
        opt.copy_inside = true;
        opt.content_only = true;
        return match fs_extra::dir::copy(blob_ver_path, &config.real_path, &opt) {
            Ok(_) => Ok(()),
            Err(e) => {
                util::crit_message(&format!("Failed to copy: {}", e));
                return MeldDirCopyFailedSnafu {
                    error_msg: e.to_string(),
                }
                .fail();
            }
        };
    }
}

pub struct Config {
    pub is_dir: bool,
    pub real_path: String,
    pub store_path: String,
    pub version: i32,
    pub versions: HashMap<i32, String>,
    pub subset: String,
    pub tag: String,
    pub blob_name: String,
    pub content_hash: String,
    pub bin: Bin,
}

impl Config {
    // TODO: actually do file mapping based on config files
    fn translate_stored_path(path: &str) -> (String, String) {
        let real_path = path_clean::clean(path);
        let store_path = real_path.clone();
        return (real_path, store_path);
    }

    pub fn get_blob_version_path(&self, version: String) -> Result<String, MeldError> {
        let blob_ver_path = format!("{}/blobs/{}/{}", self.bin.path, self.blob_name, version);
        if !util::path_exists(&blob_ver_path) {
            return BlobNotFoundSnafu.fail();
        }

        return Ok(blob_ver_path);
    }

    // Hash the Stored Path for use as an ID in tracked
    pub fn hash_path(path: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(path.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // Store the Hash Contents for use as ID in versions
    pub fn hash_contents(path: &str) -> String {
        if util::is_dir(path) {
            return String::from("DIR");
        }
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha512::new();
        std::io::copy(&mut file, &mut hasher).unwrap();
        format!("{:x}", hasher.finalize())
    }

    // Return a list of updates we need to make for the config
    pub fn get_update_type(&mut self) -> UpdateType {
        // if the config does not exist, 0 will be returned
        let config_exists = self.bin.config_exists(&self.blob_name);
        let update_subset = config_exists && self.bin.is_update_subset_needed(self);
        let update_content = config_exists && self.bin.is_update_content_needed(self);

        if config_exists {
            self.version = self.bin.get_cur_version(self) + 1;
        } else {
            self.version = 1;
        }

        if !self.is_dir {
            // If config exists, determine what needs updating
            return if !config_exists {
                UpdateType::NewConfig
            } else if update_subset && update_content {
                UpdateType::UpdateAll
            } else if update_subset {
                UpdateType::UpdateSubset
            } else if update_content {
                UpdateType::UpdateContent
            } else {
                UpdateType::NoUpdate
            };
        } else {
            return if !config_exists {
                UpdateType::NewConfig
            } else if update_subset {
                UpdateType::UpdateSubset
            } else {
                UpdateType::UpdateContent
            };
        }
    }

    pub fn new(
        path: String,
        subset: String,
        tag: String,
        bin: Bin,
        is_pull: bool,
    ) -> Result<Self, MeldError> {
        if !bin.is_valid() {
            return InvalidBinSnafu.fail();
        }

        // Get real path and store path
        let (real_path, store_path) = Config::translate_stored_path(&path);
        let blob_name = Config::hash_path(&store_path);

        // verify config exists; either in file in push or db if pull
        if !is_pull {
            if !util::path_exists(&real_path) {
                return InvalidConfigSnafu.fail();
            }
        } else {
            if !bin.config_exists(&blob_name) {
                return InvalidConfigSnafu.fail();
            }
        }

        // Maybe used later
        let versions: HashMap<i32, String> = HashMap::new();

        let mut temp = Config {
            is_dir: false,
            tag: tag,
            subset: subset,
            real_path: real_path.to_string(),
            store_path: store_path.to_string(),
            version: 0,
            versions: versions,
            blob_name: blob_name,
            content_hash: "".to_string(),
            bin: bin,
        };

        // Determine if config is a file or dir
        if !is_pull {
            temp.is_dir = util::is_dir(&temp.real_path);
            temp.content_hash = Config::hash_contents(&temp.real_path);
        } else {
            temp.is_dir = temp.bin.config_is_dir(&temp);
            if util::path_exists(&temp.real_path) {
                temp.content_hash = Config::hash_contents(&temp.real_path);
            } else {
                temp.content_hash = "".to_string();
            }
        }

        return Ok(temp);
    }
}
