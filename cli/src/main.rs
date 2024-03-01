use args::{Args, SubCommand};
use clap::Parser;
use command::{CopyArguments, UploadArguments};

mod args;
mod client;
mod command;
mod parse;
mod util;

#[tokio::main]
async fn main() {
  let args = Args::parse();
  match args.cmd {
    SubCommand::Ping => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      command::ping(server_addr).await
    }
    SubCommand::Upload {
      code_length,
      progress_bar,
      expire,
      allow_manual_deletion,
      max_download,
      output,
      source_file,
      key_nonce,
    } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      let args = UploadArguments {
        server_addr,
        auth: args.auth,
        code_length,
        progress_bar,
        expire,
        allow_manual_deletion,
        max_download,
        output,
        source_file,
        key_nonce,
      };
      command::upload(args).await;
    }
    SubCommand::Copy {
      code_length,
      expire,
      allow_manual_deletion,
      max_download,
      output,
      key_nonce,
    } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      let args = CopyArguments {
        server_addr,
        auth: args.auth,
        code_length,
        expire,
        allow_manual_deletion,
        max_download,
        output,
        key_nonce,
      };
      command::copy(args).await;
    }
    SubCommand::Download {
      progress_bar,
      url_path,
      destination,
      key_nonce,
    } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      command::download(
        server_addr,
        args.auth,
        progress_bar,
        url_path,
        destination,
        key_nonce,
      )
      .await;
    }
    SubCommand::Info { url_path } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      command::info(server_addr, url_path, args.auth).await
    }
    SubCommand::Delete { url_path } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      command::delete(server_addr, url_path, args.auth).await
    }
    SubCommand::Encrypt {
      source_file,
      destination,
      key_nonce,
    } => {
      if destination.is_file() && destination == source_file {
        panic!("Destination file has an invalid path.")
      }
      command::encrypt_file(&key_nonce, &source_file, destination).await;
    }
    SubCommand::Decrypt {
      source_file,
      destination,
      key_nonce,
    } => {
      if destination.is_file() && destination == source_file {
        panic!("Destination file has an invalid path.")
      }
      command::decrypt_file(&key_nonce, &source_file, destination).await;
    }
  };
}
