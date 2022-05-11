use rusqlite::{params, Connection};
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

pub fn hash_contents(path: &str) -> String {
    if is_dir(path) {
        return String::from("DIR");
    }
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Sha512::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
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

pub fn is_update_needed(db: &str, blob_name: &str, content_hash: &str) -> bool {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con
        .prepare("SELECT id FROM versions WHERE sphash = ? ORDER BY ver DESC")
        .unwrap();
    let mut rows = stmt.query(params![blob_name]).unwrap();

    let stored_content_hash: String = rows.next().unwrap().unwrap().get(0).unwrap();

    // no updated needed if hashes match
    return !(stored_content_hash == content_hash);
}

pub fn config_exists(db: &str, blob_name: &str) -> bool {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con.prepare("SELECT * FROM tracked WHERE id = ?").unwrap();

    return match stmt.exists(params![blob_name]) {
        Ok(b) => b,
        Err(e) => {
            error_message(&e.to_string());
            return false;
        }
    };
}

pub fn get_cur_version(db: &str, blob_name: &str) -> i32 {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con
        .prepare("SELECT ver FROM versions WHERE sphash = ? ORDER BY ver DESC")
        .unwrap();
    let mut rows = stmt.query(params![blob_name]).unwrap();

    let last_ver: i32 = rows.next().unwrap().unwrap().get(0).unwrap();

    return last_ver;
}

pub fn get_next_version(db: &str, blob_name: &str) -> i32 {
    return get_cur_version(db, blob_name) + 1;
}

// If the tracked config is a dir, content hash cannot be taken so ID will be "DIR"
pub fn config_is_dir(db: &str, blob_name: &str) -> bool {
    let con = match Connection::open(&db) {
        Ok(con) => con,
        Err(e) => {
            crit_message(&e.to_string());
            std::process::exit(1);
        }
    };

    let mut stmt = con
        .prepare("SELECT id FROM versions WHERE sphash = ? ORDER BY ver DESC")
        .unwrap();
    let mut rows = stmt.query(params![blob_name]).unwrap();

    let id: String = rows.next().unwrap().unwrap().get(0).unwrap();

    return id == "DIR";
}
