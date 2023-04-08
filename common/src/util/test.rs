use std::{collections::HashMap, hash::Hash};

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
