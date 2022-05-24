use log::info;
use log::warn;

use crate::Bin;
use crate::Database;
use crate::Error;

use std::{fs::DirBuilder, path::PathBuf};

impl Bin {
    fn is_valid(&self) -> bool {
        return self.path.exists()
            && self.blobs.exists()
            && self.maps.exists()
            && self.db.path.exists();
    }

    /// Parse a Meld Bin from a Path
    pub fn from(path: String) -> Result<Self, Error> {
        info!("Opening bin at {}", path);
        let bin = Bin {
            path: PathBuf::from(&path),
            maps: PathBuf::from(format!("{}/maps", &path)),
            blobs: PathBuf::from(format!("{}/blobs", &path)),
            db: Database {
                path: PathBuf::from(format!("{}/db.sqlite", &path)),
            },
        };

        // sanity check creation
        return if bin.is_valid() {
            Ok(bin)
        } else {
            Err(Error::InitFailed {
                msg: "Failed to Open Valid Bin".to_string(),
            })
        };
    }

    // Helper function for repeated dir creation
    fn create_dir(dirb: &DirBuilder, path: &PathBuf) -> Result<(), Error> {
        info!("Creating {:?}", path);
        match dirb.create(path) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(Error::ParentsDontExist);
                } else {
                    return Err(Error::InitFailed {
                        msg: "Failed to Create Valid Bin".to_string(),
                    });
                }
            }
        }
    }

    /// Create and init a new Meld Bin
    pub fn new(path: String, force: bool, parents: bool) -> Result<Self, Error> {
        info!("Creating bin at {}", path);
        let bin = Bin {
            path: PathBuf::from(&path),
            maps: PathBuf::from(format!("{}/maps", &path)),
            blobs: PathBuf::from(format!("{}/blobs", &path)),
            db: Database {
                path: PathBuf::from(format!("{}/db.sqlite", &path)),
            },
        };

        // Create dirbuilder and set options
        let mut dirb = DirBuilder::new();
        dirb.recursive(parents);
        if bin.path.exists() {
            if !force {
                return Err(Error::BinAlreadyExists { bin: path });
            } else {
                warn!("Removing {}", path);
                match std::fs::remove_dir_all(path) {
                    Ok(_) => {}
                    Err(e) => return Err(Error::InitFailed { msg: e.to_string() }),
                }
            }
        }

        // create all needed folders
        Bin::create_dir(&dirb, &bin.path)?;
        Bin::create_dir(&dirb, &bin.maps)?;
        Bin::create_dir(&dirb, &bin.blobs)?;

        // create and initialize SQLite table
        bin.db.create_db_schema()?;

        // sanity check creation
        return if bin.is_valid() {
            Ok(bin)
        } else {
            Err(Error::InitFailed {
                msg: "Failed to Create Valid Bin".to_string(),
            })
        };
    }
}
