use config::Environment;

pub fn get_env_source(prefix: &str) -> Environment {
  config::Environment::with_prefix(prefix)
    .prefix_separator("__")
    .separator("__")
}