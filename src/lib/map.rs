use log::info;
use sha2::{Digest, Sha512};
use walkdir::WalkDir;

use crate::hash_path;
use crate::mapper;
use crate::Config;
use crate::Error;
use crate::Map;

impl Map {
    // Getters
    pub fn get_blob(&self) -> &String {
        &self.blob
    }

    /// Add vec of configs to the map
    fn build_configs(
        path: &str,
        subset: String,
        family: String,
        tag: &String,
    ) -> Result<Vec<Config>, Error> {
        let mut configs: Vec<Config> = Vec::new();

        for entity in WalkDir::new(path) {
            match entity {
                Ok(e) => {
                    let map_path =
                        mapper::real_path_to_map(&e.path().to_str().unwrap().to_string())?;
                    configs.push(Config::from(
                        e.path().to_str().unwrap().to_string(),
                        map_path,
                        subset.clone(),
                        family.clone(),
                        tag.clone(),
                    )?);
                }
                Err(_) => (),
            }
        }
        return Ok(configs);
    }

    /// Calculate the hash of all blobs in the vec
    /// This will be compared to the stored hash to find if map update needed
    fn get_map_hash(configs: &Vec<Config>) -> String {
        let mut hasher = Sha512::new();
        for c in configs {
            hasher.update(c.get_hash());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Create a Map from a path and arguments
    pub fn new(path: &String, subset: String, family: String, tag: String) -> Result<Self, Error> {
        info!("Building map for {}", path);

        // generate variables for the new map
        let map_blob = hash_path(&path);
        let config_vec = Map::build_configs(&path, subset, family, &tag)?;
        let map_hash = Map::get_map_hash(&config_vec);

        return Ok(Map {
            blob: map_blob,
            ver: 0,
            hash: map_hash,
            tag: tag,
            configs: config_vec,
        });
    }
}
