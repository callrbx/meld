use log::error;
use snafu::{self, Snafu};
use std::{collections::HashMap, fs, path::PathBuf};

mod bin;
mod config;
mod db;
pub mod mapper;
mod version;

#[derive(Debug, Snafu)]
pub enum Error {
    // Init Related Errors
    #[snafu(display("{bin} already exists"))]
    BinAlreadyExists { bin: String },
    #[snafu(display("bin's parent tree does not exist; -p"))]
    ParentsDontExist,
    #[snafu(display("Init Failed: {msg}"))]
    InitFailed { msg: String },
    //SQL Errors
    #[snafu(display("SQL Failed: {msg}"))]
    SQLError { msg: String },
    // General Errors
    #[snafu(display("IO Error: {msg}"))]
    IOError { msg: String },
}

pub struct Database {
    path: PathBuf,
}

pub struct Bin {
    path: PathBuf,
    maps: PathBuf,
    blobs: PathBuf,
    pub db: Database,
}

pub struct Config {
    blob: String,
    real_path: String,
    subset: String,
    family: String,
    map_path: String,
    tag: String,
    is_tcl: bool,
    parent: String,
    pub versions: HashMap<String, Version>,
}

pub struct Version {
    pub data_hash: String,
    pub ver: u32,
    pub tag: String,
    pub owner: String,
}

pub fn is_dir(path: &str) -> Result<bool, Error> {
    match fs::metadata(path) {
        Err(e) => {
            error!("Could not get metadata for {}", path);
            Err(Error::IOError { msg: e.to_string() })
        }
        Ok(md) => Ok(md.is_dir()),
    }
}
