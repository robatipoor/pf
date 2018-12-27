use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn read_file(path: &Path) -> String {
    let mut file: File = File::open(path).expect("can't open file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("can't read file");
    buf
}

pub fn write_file(path: &Path, data: &str) {
    let mut file: File = File::create(path).expect("can't create file");
    file.write(data.as_bytes()).expect("can't write to file");
}
