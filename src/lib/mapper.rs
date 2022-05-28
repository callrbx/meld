use std::fs;

use log::info;

use crate::Error;

// TODO: actually do file mapping
pub fn real_path_to_map(path: &String) -> Result<String, Error> {
    // if the file doesnt exist, it cannot be cannonicalized
    // make a best guess, strip, and look for it based in the current folder
    if !crate::exists(path) {
        let clean = path_clean::clean(path);
        let mut cur_dir = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        cur_dir.push('/');
        cur_dir.push_str(&clean);
        info!("File does not exist: best guess - {}", cur_dir);
        return Ok(cur_dir);
    }

    info!("mapping: {}", path);
    //return Ok(path_clean::clean(&path));
    return Ok(fs::canonicalize(&path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string());
}

// TODO: actually do file mapping
pub fn map_to_real_path(path: &String) -> Result<String, Error> {
    Ok(path.to_string())
}
