use crate::Database;
use crate::Error;
use log::info;
use rusqlite::{params, Connection};

const INIT_TRACKED: &str = "CREATE TABLE configs (id TEXT, subset TEXT, family TEXT, map_path TEXT, is_tlc INTEGER, parent TEXT)";
const INIT_VERSIONS: &str = "CREATE TABLE versions (id TEXT, ver INTEGER, tag TEXT, owner TEXT)";

impl Database {
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
}
