use clap::{Parser, Subcommand};

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
    #[clap(short, long)]
    auth: Option<String>,
    #[clap(default_value_t = 4, short, long)]
    code_length: u32,
    #[clap(default_value_t = 7200, short, long)]
    expire_time: u32,
    #[clap(default_value_t = true, short, long)]
    deleteable: bool,
  },
  Delete {
    #[clap(short, long)]
    auth: Option<String>,
  },
  Info {
    #[clap(short, long)]
    auth: Option<String>,
  },
  Download {
    #[clap(short, long)]
    auth: Option<String>,
  },
}

fn main() {
  let args = Args::parse();
}
