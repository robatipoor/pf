use crate::*;
use std::fs;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn write_read_file_test() {
    let p = Path::new("test");
    utils::write_file(p, "Hi this is a test file");
    assert_eq!(utils::read_file(p), "Hi this is a test file");
    fs::remove_file(p).expect("remove test file unsuccessful");
}
#[test]
fn post_get_file_test() {
    assert_eq!(
        request::fetch_file(&request::create_file("hi this is test file".to_owned()).unwrap())
            .unwrap(),
        "hi this is test file"
    )
}
#[test]

fn del_file_test() {
    let re = request::create_file("hi this is test file".to_owned()).unwrap();
    request::delete_file(&re).unwrap();
}

#[test]
fn is_url_test() {
    assert_eq!(utils::is_url("https://google.com/"), true);
}
#[test]
fn is_valid_file_test() {
    let tmp_file: NamedTempFile = NamedTempFile::new().unwrap();
    assert_eq!(utils::is_valid_path_file(tmp_file.path()), true);
    assert_eq!(
        utils::is_valid_path_file(Path::new("/blabla/foo/bar/ara")),
        false
    );
}
#[test]
fn is_valid_dir_test() {
    let tmp = tempdir().unwrap();
    assert_eq!(utils::is_valid_directory(tmp.as_ref()), true);
    assert_eq!(
        utils::is_valid_directory(Path::new("/blabla/foo/bar")),
        false
    );
}
#[test]
fn path_url_test() {
    assert_eq!(utils::file_name_url("https://paste.rs/Erd"), "Erd");
}
