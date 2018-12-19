use super::*;
use tempfile::{tempdir, NamedTempFile};

#[test]
#[cfg(target_os="linux")]
fn write_read_file_test() {
    let tmp_file: NamedTempFile = NamedTempFile::new().unwrap();
    write_file(tmp_file.path(), "Hi this is a test file");
    assert_eq!(read_file(tmp_file.path()), "Hi this is a test file");
}
#[test]
fn post_get_file_test() {
    assert_eq!(
        get_file(&post_file("hi this is test file".to_owned()).unwrap()).unwrap(),
        "hi this is test file"
    )
}
#[test]

fn del_file_test() {
    let re = post_file("hi this is test file".to_owned()).unwrap();
    assert_eq!(del_file(&re).status().is_success(), true);
}

#[test]
fn is_url_test() {
    assert_eq!(is_url("https://google.com/"), true);
}
#[test]
fn is_valid_file_test() {
    let tmp_file: NamedTempFile = NamedTempFile::new().unwrap();
    assert_eq!(is_valid_file_path(tmp_file.path()), true);
    assert_eq!(is_valid_file_path(Path::new("/blabla/foo/bar/ara")), false);
}
#[test]
fn is_valid_dir_test() {
    let tmp = tempdir().unwrap();
    assert_eq!(is_valid_dir(tmp.as_ref()), true);
    assert_eq!(is_valid_dir(Path::new("/blabla/foo/bar")), false);
}
#[test]
fn path_url_test() {
    assert_eq!(path_url("https://paste.rs/Erd"), "Erd");
}
