use snafu::{self, Snafu};
use std::path::PathBuf;

mod bin;
mod db;

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
}

pub struct Database {
    path: PathBuf,
}

pub struct Bin {
    path: PathBuf,
    maps: PathBuf,
    blobs: PathBuf,
    db: Database,
}
