use crate::{utils, PastFile};
use std::fs;
use std::path::Path;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn write_read_file_test() {
    let p = Path::new("test");
    utils::write_file(p, "Hi this is a test file").unwrap();
    assert_eq!(utils::read_file(p).unwrap(), "Hi this is a test file");
    fs::remove_file(p).expect("remove test file unsuccessful");
}
#[test]
fn post_get_file_test() {
    assert_eq!(
        PastFile::fetch(&PastFile::create("hi this is test file".to_owned()).unwrap()).unwrap(),
        "hi this is test file"
    )
}
#[test]

fn del_file_test() {
    let re = PastFile::create("hi this is test file".to_owned()).unwrap();
    PastFile::delete(&*re).unwrap();
}

#[test]
fn is_url_test() {
    assert_eq!(utils::is_valid_url("https://google.com/"), true);
}
#[test]
fn is_valid_file_test() {
    let tmp_file: NamedTempFile = NamedTempFile::new().unwrap();
    assert_eq!(utils::path_exist(tmp_file.path()), true);
    assert_eq!(utils::path_exist("/blabla/foo/bar/ara"), false);
}
#[test]
fn is_valid_dir_test() {
    let tmp = tempdir().unwrap();
    assert_eq!(utils::path_exist(tmp.as_ref()), true);
    assert_eq!(utils::path_exist("/blabla/foo/bar"), false);
}
#[test]
fn path_url_test() {
    assert_eq!(utils::path_url("https://paste.rs/Erd").unwrap(), "Erd");
}
