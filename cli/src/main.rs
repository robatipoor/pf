use anyhow::anyhow;
use clap::builder::TypedValueParser as _;
use clap::{Parser, Subcommand};
use sdk::{client::PasteFileClient, result::ApiResponseResult};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  url: String,

  #[clap(subcommand)]
  cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  HealthCheck,
  Upload {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
    #[clap(default_value_t = 4, short, long)]
    code_length: u32,
    #[clap(default_value_t = 7200, short, long)]
    expire_time: u32,
    #[clap(default_value_t = true, short, long)]
    deleteable: bool,
  },
  Delete {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
  },
  Info {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
  },
  Download {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let url = "";
  let path = "";
  let client = PasteFileClient::new(&args.url);
  match args.cmd {
    SubCommand::HealthCheck => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Upload {
      auth,
      code_length,
      expire_time,
      deleteable,
    } => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Delete { auth } => {
      let (_, resp) = client.delete(path, auth).await?;

      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Info { auth } => {
      let (_, resp) = client.info(path, auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Download { auth } => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
  }

  Ok(())
}

use std::error::Error;

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
  T: std::str::FromStr,
  T::Err: Error + Send + Sync + 'static,
  U: std::str::FromStr,
  U::Err: Error + Send + Sync + 'static,
{
  let pos = s
    .find(':')
    .ok_or_else(|| format!("invalid username:password: no `:` found in `{s}`"))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
