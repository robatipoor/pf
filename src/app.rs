use clap::{App, Arg, ArgMatches};
use std::path::PathBuf;

#[derive(Debug)]
pub enum RequestMode {
    FETCH(String),
    CREATE(String),
    DELETE(String),
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub request_mode: Option<RequestMode>,
    pub output: Option<PathBuf>,
    pub log: bool,
    pub option: Option<String>,
}

pub fn get_app_args() -> AppArgs {
    let matches: ArgMatches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("fetch")
                .short("f")
                .long("fetch")
                .value_name("URL")
                .help("Sets a url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("create")
                .short("c")
                .long("create")
                .value_name("PATHFILE")
                .help("Sets a path text file")
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
                .value_name("PATHFILE")
                .help("Sets a path file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log")
                .short("l")
                .long("log")
                .help("read logs")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("option")
                .value_name("PATHFILE/URL")
                .help("Sets a path file or url")
                .takes_value(true),
        )
        .get_matches();
    let mut app_args: AppArgs = AppArgs::default();
    if matches.is_present("log") {
        app_args.log = true;
        return app_args;
    }
    if let Some(u) = matches.value_of("fetch") {
        app_args.request_mode = Some(RequestMode::FETCH(u.to_owned()));
    } else if let Some(p) = matches.value_of("create") {
        app_args.request_mode = Some(RequestMode::CREATE(p.to_owned()));
    } else if let Some(u) = matches.value_of("delete") {
        app_args.request_mode = Some(RequestMode::DELETE(u.to_owned()));
    } else if let Some(x) = matches.value_of("option") {
        app_args.option = Some(x.to_owned());
    }
    if let Some(o) = matches.value_of("output") {
        app_args.output = Some(PathBuf::from(o));
    }
    return app_args;
}
