use std::fs::{self, File};
use std::io::{self,Read, Write};
use std::path::{Path, PathBuf};
use url::Url;


pub fn read_stdin() -> Option<String> {
    let mut re = String::new();
    return io::stdin().read_to_string(&mut re).ok().map(|_| re);
}

pub fn read_file<T: AsRef<Path>>(path: T) -> String {
    let mut file: File = File::open(path).expect("failed open file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("failed read file");
    buf
}

pub fn write_file<T: AsRef<Path>>(path: T, data: &str) {
    let mut file: File = File::create(path).expect("failed create file");
    file.write(data.as_bytes()).expect("failed write to file");
}

pub fn is_valid_path_file<T: AsRef<Path>>(path: T) -> bool {
    if path.as_ref().exists() && fs::metadata(path).unwrap().is_file() {
        return true;
    } else {
        return false;
    }
}

pub fn is_valid_directory<T: AsRef<Path>>(path: T) -> bool {
    if path.as_ref().exists() && fs::metadata(path).unwrap().is_dir() {
        return true;
    } else {
        return false;
    }
}

pub fn path_in_home_dir(file_name: &str) -> PathBuf {
    dirs::home_dir().unwrap().join(Path::new(file_name))
}

pub fn is_url(s: &str) -> bool {
    match Url::parse(s) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn file_name_url(s: &str) -> String {
    Url::parse(s).unwrap().path()[1..].to_owned()
}
