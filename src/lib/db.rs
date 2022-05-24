use std::collections::HashMap;

use crate::Config;
use crate::Database;
use crate::Error;
use crate::Version;
use log::info;
use rusqlite::{params, Connection};

const INIT_TRACKED: &str = "CREATE TABLE configs (id TEXT, subset TEXT, family TEXT, map_path TEXT, is_tlc INTEGER, parent TEXT)";
const INIT_VERSIONS: &str = "CREATE TABLE versions (id TEXT, ver INTEGER, tag TEXT, owner TEXT)";

impl Database {
    // TODO: Impliment me
    pub(crate) fn is_valid(&self) -> bool {
        true
    }

    // Initialize new DB Schema
    pub(crate) fn create_db_schema(&self) -> Result<(), Error> {
        info!("Creating {:?}", self.path);
        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        match con.execute(INIT_TRACKED, params![]) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        match con.execute(INIT_VERSIONS, params![]) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        return Ok(());
    }

    // Get a map of all versions matching blob
    pub fn get_versions(&self, owner: &String) -> Result<HashMap<String, Version>, Error> {
        info!("Finding all versions with owner {}", &owner);

        let mut versions: HashMap<String, Version> = HashMap::new();

        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // select all rows from versions with matching owner
        let mut stmt = match con.prepare("SELECT * FROM versions where owner = ? ") {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // convert the rows into a MappedRows iterator
        let versions_iter = match stmt.query_map(params![owner], |row| {
            Ok(Version {
                data_hash: row.get(0)?,
                ver: row.get(1)?,
                tag: row.get(2)?,
                owner: row.get(3)?,
            })
        }) {
            Ok(i) => i,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // map the rows iterator into our hashmap
        for version in versions_iter {
            match version {
                Ok(v) => versions.insert(v.data_hash.clone(), v),
                _ => None,
            };
        }

        return Ok(versions);
    }

    // Get the current version of the config
    pub fn get_current_version(&self, owner: &String) -> Result<Option<Version>, Error> {
        info!("Finding the current version with owner {}", &owner);

        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // highest version number with matching owner
        let mut stmt = match con.prepare("SELECT * FROM versions where owner = ? ORDER BY ver DESC")
        {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // convert the rows into a MappedRows iterator
        let mut versions_iter = match stmt.query_map(params![owner], |row| {
            Ok(Version {
                data_hash: row.get(0)?,
                ver: row.get(1)?,
                tag: row.get(2)?,
                owner: row.get(3)?,
            })
        }) {
            Ok(i) => i,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        return match versions_iter.next() {
            Some(v) => match v {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(Error::SQLError { msg: e.to_string() }),
            },
            None => Ok(None),
        };
    }

    // Add a new version to the versions table
    pub fn add_config(&self, c: &Config) -> Result<(), Error> {
        info!("Adding config {}", c.get_blob());

        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // determine if config is a TLC
        let is_tlc = if c.is_tcl { 1 } else { 0 };

        // Insert config into DB configs table
        match con.execute(
            "INSERT INTO configs (id, subset, family, map_path, is_tlc, parent) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![c.blob, c.subset, c.family, c.map_path, is_tlc, c.parent],
        ) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        return Ok(());
    }

    // Add a new version to the versions table
    pub fn add_version(&self, v: &Version) -> Result<(), Error> {
        info!("Adding version {}", v.data_hash);

        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // Insert version into DB versions table
        match con.execute(
            "INSERT INTO versions (id, ver, tag, owner) VALUES (?1, ?2, ?3, ?4)",
            params![v.data_hash, v.ver, v.tag, v.owner],
        ) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        return Ok(());
    }

    // Update a version's tag
    pub fn update_version_tag(&self, v: &Version, tag: &String) -> Result<(), Error> {
        info!("Updating version tag '{}' -> '{}'", v.tag, tag);

        let con = match Connection::open(&self.path) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        // Insert version into DB versions table
        match con.execute(
            "UPDATE versions SET tag=?1 WHERE owner = ?2 AND ver = ?3",
            params![tag, v.owner, v.ver],
        ) {
            Ok(c) => c,
            Err(e) => return Err(Error::SQLError { msg: e.to_string() }),
        };

        return Ok(());
    }
}
