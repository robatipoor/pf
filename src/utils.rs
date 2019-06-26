use crate::errors::*;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use url::Url;

#[macro_export]
macro_rules! fatal {
    ($msg:tt) => {{
        eprintln!("{} in file {} line {}", $msg, file!(), line!());
        clean();
        std::process::exit(1)
    }};
}

pub fn read_from_stdin() -> Result<String> {
    std::io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .map_err(|e| {
            eprintln!("{}", e);
            Error::StdinError
        })
}

pub fn remove_file<P: AsRef<Path>>(p: P) -> Result {
    if p.as_ref().exists() {
        std::fs::remove_file(p).map_err(|e| {
            eprintln!("{}", e);
            Error::RemoveFileError
        })
    } else {
        Err(Error::FileNotExistError)
    }
}

pub fn read_file<P: AsRef<Path>>(p: P) -> Result<String> {
    File::open(p)
        .map_err(|e| {
            eprintln!("open file error {}", e);
            Error::OpenFileError
        })
        .and_then(|mut f: File| {
            let mut buf = String::new();
            f.read_to_string(&mut buf).map_err(|e| {
                eprintln!("read to string error {}", e);
                Error::ReadToStringError
            })?;
            Ok(buf)
        })
}

pub fn write_file<P: AsRef<Path>>(path: P, s: &str) -> Result {
    let mut file: File = File::create(path).map_err(|e| {
        eprintln!("{}", e);
        Error::CreateFileError
    })?;
    file.write_all(s.as_bytes()).map_err(|e| {
        eprintln!("{}", e);
        Error::WriteToFileError
    })?;
    Ok(())
}

pub fn file_exist<T: AsRef<Path>>(path: T) -> bool {
    if fs::metadata(&path).is_ok() {
        fs::metadata(path).unwrap().is_file()
    } else {
        false
    }
}

pub fn path_exist<T: AsRef<Path>>(path: T) -> bool {
    fs::metadata(path).is_ok()
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn is_valid_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}

pub fn path_url(s: &str) -> Result<String> {
    let path = Url::parse(s)
        .map_err(|e| {
            eprintln!("{}", e);
            Error::ParseError
        })?
        .path()[1..]
        .to_owned();
    Ok(path)
}
