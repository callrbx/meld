use sha2::{Digest, Sha512};
use std::fs;

pub fn crit_message(msg: &str) {
    eprintln!("[!] {}", msg);
}

pub fn error_message(msg: &str) {
    eprintln!("[-] {}", msg);
}

pub fn info_message(msg: &str) {
    println!("[*] {}", msg);
}

pub fn good_message(msg: &str) {
    println!("[+] {}", msg);
}

pub fn is_dir(path: &str) -> bool {
    match fs::metadata(path) {
        Err(_) => {
            crit_message(&format!("could not get metadata for {}", path));
            std::process::exit(1);
        }
        Ok(md) => return md.is_dir(),
    }
}

pub fn hash_path(path: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(path.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn valid_meld_dir(path: &str) -> bool {
    let bin = path;
    let db = format!("{}/meld.db", bin);
    let blobs = format!("{}/blobs/", bin);

    return path_exists(bin) && path_exists(&db) && path_exists(&blobs);
}
