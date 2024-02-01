use anyhow::anyhow;
use base64::Engine;
use clap::{Parser, Subcommand};
use sdk::{
  client::PasteFileClient, dto::request::UploadQueryParam, result::ApiResponseResult,
  util::base64::BASE64_ENGIN,
};
use std::{error::Error, path::PathBuf};
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  server_addr: String,
  #[clap(short, long, value_parser = parse_auth)]
  auth: Option<(String, String)>,
  #[clap(subcommand)]
  cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  Ping,
  Upload {
    #[clap(short, long)]
    code_length: Option<usize>,
    #[clap(short, long,value_parser = parse_expire_time_from_str)]
    expire: Option<u64>,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(short, long)]
    delete_manually: Option<bool>,
    #[clap(default_value_t = false, short, long)]
    qrcode: bool,
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long)]
    source_file: PathBuf,
  },
  Delete {
    #[arg(short, long)]
    url_path: String,
  },
  Info {
    #[arg(short, long)]
    url_path: String,
  },
  Download {
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[arg(short, long)]
    url_path: String,
    #[clap(short, long)]
    destination_dir: PathBuf,
  },
}

fn parse_expire_time_from_str(
  expire_time: &str,
) -> Result<u64, Box<dyn Error + Send + Sync + 'static>> {
  let words: Vec<&str> = expire_time.split_whitespace().collect();
  if words.len() != 2 {
    return Err("Invalid expire time format".into());
  }
  let value: u64 = words[0].parse()?;
  let multiplier = match words[1].to_lowercase().as_str() {
    "second" | "sec" | "s" => value,
    "minute" | "min" => value * 60,
    "hour" | "h" => value * 3600,
    "day" | "d" => value * 3600 * 24,
    "month" => value * 3600 * 24 * 30,
    "year" | "y" => value * 3600 * 24 * 30 * 12,
    _ => return Err("Invalid expire time format".into()),
  };
  Ok(value * multiplier)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let client = PasteFileClient::new(args.server_addr);
  match args.cmd {
    SubCommand::Ping => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", resp.message);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Upload {
      code_length,
      progress_bar,
      expire,
      delete_manually,
      max_download,
      qrcode,
      source_file,
    } => {
      let query = UploadQueryParam {
        max_download,
        code_length,
        expire_secs: expire,
        delete_manually,
      };
      let (_, resp) = if progress_bar {
        client
          .upload_with_progress_bar(&source_file, &query, args.auth)
          .await
      } else {
        client.upload_from(&source_file, &query, args.auth).await
      }?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          if qrcode {
            println!(
              "{}",
              std::str::from_utf8(&BASE64_ENGIN.decode(resp.qrcode)?)?
            );
          } else {
            println!("{}", &Url::parse(&resp.url)?.path()[1..]);
          }
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Download {
      progress_bar,
      url_path,
      destination_dir,
    } => {
      let (_, resp) = if progress_bar {
        client
          .download_with_progress_bar(&url_path, args.auth, &destination_dir)
          .await
      } else {
        client
          .download_into(&url_path, args.auth, &destination_dir)
          .await
      }?;
      match resp {
        ApiResponseResult::Ok(_) => {
          println!("Done");
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Info { url_path } => {
      let (_, resp) = client.info(&url_path, args.auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Delete { url_path } => {
      let (_, resp) = client.delete(&url_path, args.auth).await?;
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

fn parse_auth(s: &str) -> Result<(String, String), Box<dyn Error + Send + Sync + 'static>> {
  let pos = s
    .find(':')
    .ok_or_else(|| format!("invalid username:password: no `:` found in {s}"))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
