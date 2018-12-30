use clap::{App, Arg, ArgMatches};

pub fn get_arg_matches<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
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
            Arg::with_name("log")
                .short("l")
                .long("log")
                .help("get log file")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("option")
                .value_name("FILE/URL")
                .help("Sets a file or url")
                .takes_value(true),
        )
        .get_matches()
}
