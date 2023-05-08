use std::{collections::HashMap, hash::Hash};

#[macro_export]
macro_rules! assert_ok {
  ($result:expr) => {
    assert!(
      matches!($result, sdk::result::ApiResponseResult::Ok(_)),
      "match failed: {:?}",
      $result,
    )
  };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr $(, $closure:expr )?) => {
        assert!(
          matches!($result,sdk::result::ApiResponseResult::Err(ref _e) $( if $closure(_e) )?),
          "match failed: {:?}",$result,
        )
    };
}

#[macro_export]
macro_rules! unwrap {
  ($result:expr) => {
    match $result {
      sdk::result::ApiResponseResult::Ok(resp) => resp,
      sdk::result::ApiResponseResult::Err(e) => {
        panic!("called `util::unwrap!()` on an `Err` value {e:?}")
      }
    }
  };
}

pub fn eq<T>(a: &[T], b: &[T]) -> bool
where
  T: Eq + Hash,
{
  fn count<T>(items: &[T]) -> HashMap<&T, usize>
  where
    T: Eq + Hash,
  {
    let mut cnt = HashMap::new();
    for i in items {
      *cnt.entry(i).or_insert(0) += 1
    }
    cnt
  }
  count(a) == count(b)
}

pub fn vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
  a.len() == b.len() && !a.iter().zip(b.iter()).any(|(a, b)| *a != *b)
}
