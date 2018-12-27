use clap::{App, Arg, ArgMatches};

pub fn get_arg_matches<'a>() -> ArgMatches<'a> {
    App::new("pf")
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
            Arg::with_name("log")
                .short("l")
                .long("log")
                // .value_name("PATH FILE")
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
