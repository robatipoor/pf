#[macro_export]
macro_rules! assert_response_ok {
  ($result:expr) => {
    assert!(
      matches!($result, pf_sdk::dto::response::ApiResponseResult::Ok(_)),
      "match failed: {:?}",
      $result,
    )
  };
}

#[macro_export]
macro_rules! assert_response_err {
    ($result:expr $(, $closure:expr )?) => {
        assert!(
          matches!($result,pf_sdk::dto::response::ApiResponseResult::Err(ref _e) $( if $closure(_e) )?),
          "match failed: {:?}",$result,
        )
    };
}

#[macro_export]
macro_rules! unwrap {
  ($result:expr) => {
    match $result {
      pf_sdk::dto::response::ApiResponseResult::Ok(resp) => resp,
      pf_sdk::dto::response::ApiResponseResult::Err(e) => {
        panic!("called `util::unwrap!()` on an `Err` value {e:?}")
      }
    }
  };
}
