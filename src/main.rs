mod args;

use args::{AppArgs, RequestMode::*};
use pf::*;
use pf::constants::*;
use std::process;

fn main() {
    let app_args = AppArgs::get_app_args();
    conf::config_log();
    if app_args.log {
        let log = utils::read_file(utils::home_dir().unwrap().join(LOG_FILE)).unwrap();
        println!("{}", log);
        return;
    }
    if app_args.request_mode.is_none() {
        if let Some(opt) = app_args.option {
            if utils::is_valid_url(&*opt) {
                println!("{}", PastFile::fetch(&*opt).unwrap());
            } else if utils::path_exist(&*opt) {
                let link = PastFile::create(utils::read_file(opt).unwrap()).unwrap();
                println!("{}", link);
            } else {
                eprintln!("invalid args !");
                process::exit(1);
            }
            return;
        }
    }
    if let Some(rm) = app_args.request_mode {
        match rm {
            FETCH(u) => {
                if utils::is_valid_url(&*u) {
                    if let Some(o) = app_args.output {
                        if utils::path_exist(&*o) {
                            utils::write_file(
                                o.join(format!("pafi-{}", utils::path_url(&*u).unwrap())).as_path(),
                                &*PastFile::fetch(&*u).unwrap(),
                            )
                            .unwrap();
                        }
                    } else {
                        println!("{} ", &*PastFile::fetch(&*u).unwrap());
                    }
                    return;
                } else {
                    eprintln!("url not valid");
                    process::exit(1);
                }
            }
            CREATE(p) => {
                if let Ok(resp_url) = PastFile::create(utils::read_file(p).unwrap()) {
                    println!("{} ", resp_url);
                    return;
                } else {
                    eprintln!("error post file");
                    process::exit(1);
                }
            }
            DELETE(u) => {
                if let Ok(o) = PastFile::delete(&*u) {
                    println!("{} ", o);
                    return;
                } else {
                    eprintln!("unsuccessful delete file");
                    process::exit(1);
                }
            }
        }
    } else if let Ok(link) = PastFile::create(utils::read_from_stdin().unwrap()) {
        println!("{} ", link);
    }
}
