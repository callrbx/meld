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

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn valid_meld_dir(path: &str) -> bool {
    let bin = path;
    let db = format!("{}/meld.db", bin);
    let blobs = format!("{}/blobs/", bin);

    return path_exists(bin) && path_exists(&db) && path_exists(&blobs);
}
