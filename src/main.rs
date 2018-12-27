use log::info;
use reqwest::{Client, Response};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use url::Url;

mod app;
mod file_mod;
mod log_mod;
#[cfg(test)]
mod tests;
use crate::file_mod::*;

const URL_SERVICE: &str = "https://paste.rs/";

fn main() {
    let matches = app::get_arg_matches();
    log_mod::config_log();
    if let Some(u) = matches.value_of("get") {
        if is_url(u) {
            if let Some(o) = matches.value_of("output") {
                if is_valid_dir(Path::new(o)) {
                    write_file(
                        Path::new(o).join(format!("pafi-{}", path_url(u))).as_path(),
                        &get_file(u).unwrap(),
                    );
                }
            } else {
                println!("{}", get_file(u).unwrap());
            }
        } else {
            eprintln!("url not valid");
        }
    } else if let Some(p) = matches.value_of("post") {
        if let Some(resp_url) = post_file(read_file(Path::new(p))) {
            println!("{}", resp_url);
        } else {
            eprintln!("error post file");
        }
    } else if let Some(u) = matches.value_of("delete") {
        if del_file(u).status().is_success() {
            println!("{} {}", u, "Deleted !");
        } else {
            eprintln!("unsuccessful delete file");
        }
    } else if let Some(x) = matches.value_of("option") {
        if is_url(x) {
            if let Some(o) = matches.value_of("output") {
                if is_valid_dir(Path::new(o)) {
                    write_file(
                        Path::new(o).join(format!("pafi-{}", path_url(x))).as_path(),
                        &get_file(x).unwrap(),
                    );
                }
            } else {
                println!("{}", get_file(x).unwrap());
            }
        } else if is_valid_file_path(Path::new(x)) {
            if let Some(resp_url) = post_file(read_file(Path::new(x))) {
                println!("{}", resp_url);
            } else {
                eprintln!("unsuccessful post file");
            }
        }
    } else if matches.is_present("log") {
        println!("{}", read_file(log_mod::path_log_file().as_path()));
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        if let Some(resp_url) = post_file(buf) {
            println!("{}", resp_url);
        }
    }
}

fn is_url(s: &str) -> bool {
    match Url::parse(s) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn is_valid_file_path(p: &Path) -> bool {
    if p.exists() && fs::metadata(p).unwrap().is_file() {
        return true;
    } else {
        return false;
    }
}

fn is_valid_dir(p: &Path) -> bool {
    if p.exists() && fs::metadata(p).unwrap().is_dir() {
        return true;
    } else {
        return false;
    }
}

fn path_url(s: &str) -> String {
    Url::parse(s).unwrap().path()[1..].to_owned()
}

fn get_file(url: &str) -> Option<String> {
    let client = Client::new();
    let mut resp: Response = client.get(url).send().expect("can't get file");
    handle_response(&mut resp).and_then(|x| {
        info!("GET : {:?} ", url.trim());
        Some(x)
    })
}

fn post_file(data: String) -> Option<String> {
    let client = Client::new();
    let mut resp: Response = client
        .post(URL_SERVICE)
        .body(data)
        .send()
        .expect("can't post file");
    handle_response(&mut resp).and_then(|url| {
        info!("POST : {:?} ", url.trim());
        Some(url)
    })
}

fn del_file(url: &str) -> Response {
    let client = Client::new();
    client
        .delete(url)
        .send()
        .and_then(|x| {
            if x.status().is_success() {
                info!("DELETE : {:?} ", url.trim());
                return Ok(x);
            } else {
                panic!("Error !");
            }
        })
        .expect("unsuccessful delete file ")
}

fn handle_response(resp: &mut Response) -> Option<String> {
    if resp.status().is_success() {
        let mut buf = String::new();
        resp.read_to_string(&mut buf).expect("can't read buf");
        Some(buf)
    } else {
        None
    }
}
