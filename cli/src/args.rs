use clap::{Parser, Subcommand, ValueEnum};

use std::path::PathBuf;

use crate::{
  parse::{parse_auth, parse_destination, parse_expire_time, parse_key_nonce, parse_source_file},
  util::crypto::KeyNonce,
};

const HELP_ENCRYPT :&str = "The encrypt format should be `key:nonce`, with the key being 32 characters in length and the nonce being 19 characters.";
const HELP_DECRYPT :&str = "The decrypt format should be `key:nonce`, with the key being 32 characters in length and the nonce being 19 characters.";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[arg(
    short,
    long,
    help = "The server address format should be http:// or https:// followed by the IP address and port."
  )]
  pub server_addr: Option<String>,
  #[clap(short, long, value_parser = parse_auth, help = "The auth format should be `username:password`")]
  pub auth: Option<(String, String)>,
  #[clap(subcommand)]
  pub cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  Ping,
  Upload {
    #[clap(short, long)]
    code_length: Option<usize>,
    #[clap(short, long,value_parser = parse_expire_time)]
    expire: Option<u64>,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(short, long)]
    delete_manually: Option<bool>,
    #[clap(default_value_t = UploadOutput::Json, short, long)]
    out: UploadOutput,
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_ENCRYPT)]
    key_nonce: Option<KeyNonce>,
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
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_DECRYPT)]
    key_nonce: Option<KeyNonce>,
  },
  Encrypt {
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_ENCRYPT)]
    key_nonce: KeyNonce,
  },
  Decrypt {
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_DECRYPT)]
    key_nonce: KeyNonce,
  },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum UploadOutput {
  QrCode,
  Url,
  UrlPath,
  Json,
}

impl std::fmt::Display for UploadOutput {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self
      .to_possible_value()
      .expect("no values are skipped")
      .get_name()
      .fmt(f)
  }
}
