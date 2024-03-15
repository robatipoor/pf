#[macro_export]
macro_rules! assert_ok {
  ($result:expr) => {
    assert!(
      matches!($result, std::result::Result::Ok(_)),
      "match failed: {:?}",
      $result,
    )
  };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr $(, $closure:expr )?) => {
        assert!(
          matches!($result,std::result::Result::Err(ref _e) $( if $closure(_e) )?),
          "match failed: {:?}",$result,
        )
    };
}
