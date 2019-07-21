use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use clap::{Arg, App};

mod model;
mod parser;
mod processor;
mod output_formatter;

use parser::{prepare_file, file_extension_to_filetype};
use processor::{run};
use output_formatter::{format_module};

fn main() {
    let matches = App::new("fcheck")
        .version("0.3.0")
        .about("A language agnostic orchestration tool for integration and system testing.")
        .arg(Arg::with_name("config-file")
            .short("c")
            .long("config-file")
            // .value_name("FILE")
            .help("Configuration file containing tests to be run")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("report-file")
            .short("r")
            .long("report-file")
            .help("File name of output report")
            .required(false))
        // .arg(Arg::with_name("v")
        //     .short("v")
        //     .multiple(true)
        //     .help("Sets the level of verbosity"))
        .get_matches();

//         .version('0.1.0')
//   .option('-c, --config-file [file]', 'Configuration file containing tests to be run', './config/config.toml')
//   .option('-r, --report-file [file]', 'File with test configuration', './data/report.json')
//   .option('-v, --verbose-errors', 'Verbose error logging')
//   .parse(process.argv);

    let config_file = matches.value_of("config-file").unwrap_or("./config.toml");
    let output_report_path = matches.value_of("report-file").unwrap_or("./output/report.json");

    // match matches.occurrences_of("v") {
    //     0 => println!("No verbose info"),
    //     1 => println!("Some verbose info"),
    //     2 => println!("Tons of verbose info"),
    //     3 | _ => println!("Don't be crazy"),
    // }

    let config_path = Path::new(config_file);
    if !config_path.exists() {
        println!("config-file not found. (Value provided: {})", config_file);
        std::process::exit(1)
    }

    let config_file_type = 
        get_extension_from_filename(config_file)
        .and_then(|ext| file_extension_to_filetype(ext))
        .expect("Config file has invalid extension type. Valid extensions: .toml, .dhall");

    let config_contents = fs::read_to_string(config_path)
        .expect("Failed to read config file.");

    let module = prepare_file(config_file_type, config_contents)
        .expect("Failed to process config file");
    
    println!("out: {:?}", module);

    let res = run(&module);

    println!("Module Result: \n{}", format_module(&res));

    //To do
    // Add module name
    // Add full success of module

}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}
