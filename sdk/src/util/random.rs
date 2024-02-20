use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_string(len: usize) -> String {
  rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(len)
    .map(char::from)
    .collect()
}

pub fn generate_random_string_with_prefix(prefix: &str) -> String {
  format!("{prefix}_{}", generate_random_string(10))
}