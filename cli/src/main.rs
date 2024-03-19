use args::{Args, SubCommand};
use clap::Parser;
use command::{CopyArguments, UploadArguments};
use sdk::util::{
  file::{add_extension, get_content_type},
  random::generate_random_string,
};

mod args;
mod client;
mod command;
mod parse;
mod util;

const FILE_NAME_LENGTH: usize = 5;

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
      file_name,
      code_length,
      expire,
      allow_manual_deletion,
      max_download,
      output,
      key_nonce,
    } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      let stdin = tokio::io::stdin();
      let file_name = if key_nonce.is_some() {
        add_extension(
          file_name
            .unwrap_or_else(|| add_extension(generate_random_string(FILE_NAME_LENGTH), "txt")),
          "bin",
        )
      } else {
        file_name.unwrap_or_else(|| add_extension(generate_random_string(FILE_NAME_LENGTH), "txt"))
      }
      .file_name()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();

      let content_type = get_content_type(&file_name).unwrap();
      let args = CopyArguments {
        server_addr,
        auth: args.auth,
        file_name,
        content_type,
        code_length,
        expire,
        allow_manual_deletion,
        max_download,
        output,
        key_nonce,
      };
      command::copy(stdin, args).await;
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
    SubCommand::Paste {
      url_path,
      key_nonce,
    } => {
      let server_addr = args.server_addr.expect("Server address should be set.");
      let stdout: tokio::io::Stdout = tokio::io::stdout();
      command::paste(server_addr, args.auth, url_path, key_nonce, stdout).await;
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
      progress_bar,
      source_file,
      destination,
      key_nonce,
    } => {
      if destination.is_file() && destination == source_file {
        panic!("Destination file has an invalid path.")
      }
      command::encrypt_file(progress_bar, &key_nonce, &source_file, destination).await;
    }
    SubCommand::Decrypt {
      progress_bar,
      source_file,
      destination,
      key_nonce,
    } => {
      if destination.is_file() && destination == source_file {
        panic!("Destination file has an invalid path.")
      }
      command::decrypt_file(progress_bar, &key_nonce, &source_file, destination).await;
    }
  };
}
