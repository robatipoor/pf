use std::path::PathBuf;

pub fn parse_config_path_from_arguments(args: &[String]) -> Option<PathBuf> {
  args
    .iter()
    .find(|arg| arg.contains("--config"))
    .and_then(|arg| arg.split("--config").nth(1).map(|a| PathBuf::from(&a[1..])))
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_parse_config_path_from_arguments() {
    let expected = std::path::PathBuf::from("production-config.toml");
    let args = vec![
      "meta_node".to_string(),
      "--config=production-config.toml".to_string(),
    ];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
    let args = vec!["--config production-config.toml".to_string()];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
  }
}
