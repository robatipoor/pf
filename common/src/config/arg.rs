use std::path::PathBuf;

pub fn parse_config_path_from_arguments(args: &[String]) -> Option<PathBuf> {
  args
    .iter()
    .find(|arg| arg.starts_with("--config") || arg.starts_with("-c"))
    .and_then(|arg| {
      if arg.contains('=') {
        arg
          .split('=')
          .nth(1)
          .map(|c| c.trim().replace(['"', '\''], ""))
          .filter(|c| c.ends_with(".toml"))
          .map(PathBuf::from)
      } else if args.len() == 3 {
        args
          .get(2)
          .map(|c| c.trim().replace(['"', '\''], ""))
          .filter(|c| c.ends_with(".toml"))
          .map(PathBuf::from)
      } else {
        None
      }
    })
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_parse_config_path_from_valid_arguments() {
    let expected = std::path::PathBuf::from("prod-config.toml");
    let args = vec!["node".to_string(), "--config=prod-config.toml".to_string()];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
    let args = vec![
      "node".to_string(),
      "--config".to_string(),
      "prod-config.toml".to_string(),
    ];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
    let args = vec!["node".to_string(), "-c=prod-config.toml".to_string()];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
    let args = vec![
      "node".to_string(),
      "-c".to_string(),
      "prod-config.toml".to_string(),
    ];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);

    let args = vec![
      "node".to_string(),
      "-c".to_string(),
      r#""prod-config.toml""#.to_string(),
    ];
    let result = parse_config_path_from_arguments(&args).unwrap();
    assert_eq!(result, expected);
  }

  #[test]
  fn test_parse_config_path_from_invalid_arguments() {
    let args = vec!["node".to_string(), "--conf=config.toml".to_string()];
    let result = parse_config_path_from_arguments(&args);
    assert!(result.is_none());
    let args = vec!["node".to_string(), "config.toml--config".to_string()];
    let result = parse_config_path_from_arguments(&args);
    assert!(result.is_none());
    let args = vec![
      "node".to_string(),
      "--config=".to_string(),
      "config.toml".to_string(),
    ];
    let result = parse_config_path_from_arguments(&args);
    assert!(result.is_none());
    let args = vec![
      "node".to_string(),
      "--c".to_string(),
      "config.toml".to_string(),
    ];
    let result = parse_config_path_from_arguments(&args);
    assert!(result.is_none());
    let args = vec!["node".to_string(), "-c".to_string(), "foo".to_string()];
    let result = parse_config_path_from_arguments(&args);
    assert!(result.is_none());
  }
}
