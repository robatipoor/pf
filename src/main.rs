#[macro_use]
extern crate clap;
extern crate chrono;
extern crate dirs;
extern crate fern;
extern crate log;
extern crate reqwest;
extern crate url;

mod app;
mod constants;
mod utils;
mod config_log;
mod request;

use app::RequestMode::*;
use constants::*;
use std::path::Path;

#[cfg(test)]
mod tests;

fn main() {
    let app_args = app::get_app_args();
    config_log::config_log();
    if app_args.log {
        println!(
            "{}",
            utils::read_file(utils::path_in_home_dir(LOG_FILE_NAME).as_path())
        );
        return;
    }
    if let None = app_args.request_mode {
        if let Some(opt) = app_args.option {
            if utils::is_url(&*opt) {
                println!("{:?}", request::fetch_file(&*opt).unwrap());
            } else if utils::is_valid_path_file(Path::new(&*opt)) {
                println!("{:?}", request::create_file(opt).unwrap());
            } else {
                panic!("invalid args !");
            }
            return;
        }
    }

    match app_args.request_mode.unwrap() {
        FETCH(u) => {
            if utils::is_url(&*u) {
                if let Some(o) = app_args.output {
                    if utils::is_valid_directory(&*o) {
                        utils::write_file(
                            o.join(format!("pafi-{}", utils::file_name_url(&*u)))
                                .as_path(),
                            &*request::fetch_file(&*u).unwrap(),
                        );
                    }
                } else {
                    println!("{}", &*request::fetch_file(&*u).unwrap());
                }
            } else {
                panic!("url not valid");
            }
        }
        CREATE(p) => {
            if let Ok(resp_url) = request::create_file(utils::read_file(p)) {
                println!("{}", resp_url);
            } else {
                panic!("error post file");
            }
        }
        DELETE(u) => {
            if let Ok(o) = request::delete_file(&*u) {
                println!("{}  Deleted !", o,);
            } else {
                panic!("unsuccessful delete file");
            }
        }
    }
    
    if let Ok(resp_url) = request::create_file(utils::read_stdin().unwrap()) {
        println!("{}", resp_url);
    }
}
