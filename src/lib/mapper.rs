use std::fs;

use log::info;

use crate::Error;

// TODO: actually do file mapping
pub fn map_file(path: &String) -> Result<String, Error> {
    info!("mapping: {}", path);
    //return Ok(path_clean::clean(&path));
    return Ok(fs::canonicalize(&path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string());
}
