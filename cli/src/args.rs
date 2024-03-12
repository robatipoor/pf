use clap::{Parser, Subcommand, ValueEnum};
use sdk::{dto::FileUrlPath, util::crypto::KeyNonce};

use std::path::PathBuf;

use crate::parse::{
  parse_auth, parse_destination, parse_expire_time, parse_file_url_path, parse_key_nonce,
  parse_source_file,
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
  #[clap(about = "Ping the server to check connectivity")]
  Ping,
  #[clap(about = "Upload a file to the server")]
  Upload {
    #[clap(short, long)]
    code_length: Option<usize>,
    #[clap(short, long,value_parser = parse_expire_time)]
    expire: Option<u64>,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(short, long)]
    allow_manual_deletion: Option<bool>,
    #[clap(default_value_t = UploadOutput::Json, short, long)]
    output: UploadOutput,
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_ENCRYPT)]
    key_nonce: Option<KeyNonce>,
  },
  #[clap(about = "Copy text data from standard input (stdin) to the server")]
  Copy {
    #[clap(short, long)]
    file_name: Option<String>,
    #[clap(short, long)]
    code_length: Option<usize>,
    #[clap(short, long,value_parser = parse_expire_time)]
    expire: Option<u64>,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(short, long)]
    allow_manual_deletion: Option<bool>,
    #[clap(default_value_t = UploadOutput::Json, short, long)]
    output: UploadOutput,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_ENCRYPT)]
    key_nonce: Option<KeyNonce>,
  },
  #[clap(about = "Delete a file from the server")]
  Delete {
    #[arg(short, long, value_parser = parse_file_url_path)]
    url_path: FileUrlPath,
  },
  #[clap(about = "Get information about a file on the server")]
  Info {
    #[arg(short, long, value_parser = parse_file_url_path)]
    url_path: FileUrlPath,
  },
  #[clap(about = "Download a file from the server")]
  Download {
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[arg(short, long, value_parser = parse_file_url_path)]
    url_path: FileUrlPath,
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_DECRYPT)]
    key_nonce: Option<KeyNonce>,
  },
  #[clap(about = "Retrieve text data from the server and paste it to standard output (stdout)")]
  Paste {
    #[arg(short, long, value_parser = parse_file_url_path)]
    url_path: FileUrlPath,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_DECRYPT)]
    key_nonce: Option<KeyNonce>,
  },
  #[clap(about = "Encrypt a file before uploading to the server")]
  Encrypt {
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_ENCRYPT)]
    key_nonce: KeyNonce,
  },
  #[clap(about = "Decrypt a file after downloading from the server")]
  Decrypt {
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long, value_parser = parse_source_file)]
    source_file: PathBuf,
    #[clap(short, long, value_parser = parse_destination)]
    destination: PathBuf,
    #[clap(long, value_parser = parse_key_nonce, help = HELP_DECRYPT)]
    key_nonce: KeyNonce,
  },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
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
