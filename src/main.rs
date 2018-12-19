use clap::{App, Arg};
use reqwest::Client;
use reqwest::Response;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use url::Url;

#[cfg(test)]
mod test;

const URL_SERVICE: &str = "https://paste.rs/";

fn main() {
    let matches = App::new("pf")
        .version("0.1.1")
        .author("Mahdi <Mahdi.robatipoor@gmail.com>")
        .about("file sharing from the command line")
        .arg(
            Arg::with_name("get")
                .short("g")
                .long("get")
                .value_name("URL")
                .help("Sets a url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("post")
                .short("p")
                .long("post")
                .value_name("FILE")
                .help("Sets a text file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .short("d")
                .long("delete")
                .value_name("URL")
                .help("Sets a url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("PATH FILE")
                .help("Sets a path file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("option")
                .value_name("FILE/URL")
                .help("Sets a file or url")
                .takes_value(true),
        )
        .get_matches();
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
            println!("{} {}", u,"Deleted !");
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

fn read_file(path: &Path) -> String {
    let mut file: File = File::open(path).expect("can't open file");
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("can't read file");
    buf
}

fn write_file(path: &Path, data: &str) {
    let mut file: File = File::create(path).expect("can't create file");
    file.write(data.as_bytes()).expect("can't write to file");
}

fn get_file(url: &str) -> Option<String> {
    let client = Client::new();
    let mut resp: Response = client.get(url).send().expect("can't get file");
    handle_response(&mut resp)
}

fn post_file(data: String) -> Option<String> {
    let client = Client::new();
    let mut resp: Response = client
        .post(URL_SERVICE)
        .body(data)
        .send()
        .expect("can't post file");
    handle_response(&mut resp)
}

fn del_file(url: &str) -> Response {
    let client = Client::new();
    client.delete(url).send().expect("can't delete file")
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
