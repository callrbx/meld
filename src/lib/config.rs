use std::collections::HashMap;
use std::fs;

use log::info;
use log::warn;
use sha2::{Digest, Sha512};

use crate::is_dir;
use crate::Config;
use crate::Error;

impl Config {
    // Getters
    pub fn get_blob(&self) -> &String {
        &self.blob
    }

    pub fn get_real_path(&self) -> &String {
        &self.real_path
    }

    pub fn get_tag(&self) -> &String {
        &self.tag
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

    /// Create a Config from a path and arguments
    pub fn from(
        real_path: String,
        map_path: String,
        subset: String,
        family: String,
        tag: String,
    ) -> Result<Self, Error> {
        info!("Using config at {}", real_path);
        info!("Config mapped to {}", map_path);

        let config = Config {
            blob: Config::hash_path(&map_path),
            subset,
            family,
            map_path,
            is_tcl: true,
            parent: "".to_string(),
            real_path,
            tag,
            versions: HashMap::new(),
        };

        return Ok(config);
    }
}
