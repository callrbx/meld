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

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}
