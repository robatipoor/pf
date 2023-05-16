use anyhow::anyhow;
use clap::{Parser, Subcommand};
use sdk::{client::PasteFileClient, model::request::UploadParamQuery, result::ApiResponseResult};
use std::{error::Error, path::PathBuf};
use tokio::io::AsyncWriteExt;

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
    code_length: usize,
    #[clap(default_value_t = 7200, short, long)]
    expire_time: u64,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(default_value_t = true, short, long)]
    deleteable: bool,
    #[clap(short, long)]
    file: PathBuf,
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
    #[clap(short, long)]
    path: PathBuf,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let url = url::Url::parse(&args.url)?;
  let client = PasteFileClient::new(&format!("{}//{}", url.scheme(), url.host_str().unwrap()));
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
      max_download,
      file,
    } => {
      let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
      let content_type = mime_guess::from_path(&file)
        .first()
        .unwrap()
        .essence_str()
        .to_owned();
      let file = tokio::fs::read(file).await?;
      let query = UploadParamQuery {
        max_download,
        code_length: Some(code_length),
        expire_time: Some(expire_time),
        deleteable: Some(deleteable),
      };
      let (_, resp) = client
        .upload(file_name, &content_type, &query, file, auth)
        .await?;
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
      let (_, resp) = client.delete(url.path(), auth).await?;
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
      let (_, resp) = client.info(url.path(), auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Download { path, auth } => {
      let (_, resp) = client.download(url.path(), auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          let file_name = PathBuf::from(url.path())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
          let _ = tokio::fs::create_dir_all(&path).await;
          let mut file = tokio::fs::File::create(path.join(file_name)).await?;
          file.write_all(&resp).await?;
          println!("Ok");
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
  }
  Ok(())
}

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
