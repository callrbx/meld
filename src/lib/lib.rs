use log::{error, info, warn};
use sha2::{Digest, Sha512};
use snafu::{self, Snafu};
use std::{collections::HashMap, fs, path::PathBuf};

mod bin;
mod config;
mod db;
mod map;
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
    // Bin Errors
    #[snafu(display("Map Update Not Needed"))]
    UpdateNotNeeded,
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
    hash: String,
    pub versions: HashMap<String, Version>,
}

pub struct Version {
    pub data_hash: String,
    pub ver: u32,
    pub tag: String,
    pub owner: String,
}

pub struct Map {
    pub blob: String,
    pub ver: u32,
    pub hash: String,
    pub tag: String,
    pub configs: Vec<Config>,
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

// SHA512 of mapped file name
pub fn hash_path(path: &str) -> String {
    info!("Hashing mapped name: {}", path);
    let mut hasher = Sha512::new();
    hasher.update(path.as_bytes());
    format!("{:x}", hasher.finalize())
}

// SHA512 hash of file contents
pub fn hash_contents(path: &str) -> Result<String, Error> {
    if is_dir(path)? {
        warn!("not hashing {}. dir", path);
        return Ok(String::from("DIR"));
    }
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Sha512::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    Ok(format!("{:x}", hasher.finalize()))
}
