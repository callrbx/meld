use log::info;
use std::collections::HashMap;

use crate::hash_contents;
use crate::hash_path;
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

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    /// Create a Config from a path and arguments
    pub fn from(
        real_path: String,
        map_path: String,
        subset: String,
        family: String,
        tag: String,
    ) -> Result<Self, Error> {
        info!("Using config at {}", &real_path);
        info!("Config mapped to {}", map_path);

        let config = Config {
            blob: hash_path(&map_path),
            subset,
            family,
            map_path,
            hash: hash_contents(&real_path)?,
            real_path,
            tag,
            versions: HashMap::new(),
        };

        return Ok(config);
    }
}
