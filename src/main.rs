mod cli;

use cli::{AppArgs, RequestMode::*};
use pf::*;
use std::path::Path;
use std::process;

fn main() {
    let app_args = AppArgs::get_app_args();
    conf::config_log();
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
                println!("{}", PastFile::fetch(&*opt).unwrap());
            } else if utils::is_valid_path_file(Path::new(&*opt)) {
                println!("{}", PastFile::create(utils::read_file(opt)).unwrap());
            } else {
                panic!("invalid args !");
            }
            return;
        }
    }
    if let Some(rm) = app_args.request_mode {
        match rm {
            FETCH(u) => {
                if utils::is_url(&*u) {
                    if let Some(o) = app_args.output {
                        if utils::is_valid_directory(&*o) {
                            utils::write_file(
                                o.join(format!("pafi-{}", utils::file_name_url(&*u)))
                                    .as_path(),
                                &*PastFile::fetch(&*u).unwrap(),
                            );
                        }
                    } else {
                        println!("{} ", &*PastFile::fetch(&*u).unwrap());
                    }
                    process::exit(0);
                } else {
                    panic!("url not valid");
                }
            }
            CREATE(p) => {
                if let Ok(resp_url) = PastFile::create(utils::read_file(p)) {
                    println!("{} ", resp_url);
                    process::exit(0);
                } else {
                    panic!("error post file");
                }
            }
            DELETE(u) => {
                if let Ok(o) = PastFile::delete(&*u) {
                    println!("{} ", o);
                    process::exit(0);
                } else {
                    panic!("unsuccessful delete file");
                }
            }
        }
    } else {
        if let Ok(resp_url) = PastFile::create(utils::read_stdin().unwrap()) {
            println!("{} ", resp_url);
        }
    }
}
