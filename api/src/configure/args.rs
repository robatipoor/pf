use std::path::PathBuf;

#[derive(clap::Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  pub settings: Option<PathBuf>,
}
