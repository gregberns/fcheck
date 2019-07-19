use std::path::Path;
use clap::{Arg, App, SubCommand};
use toml::Value;

mod parser;

// use parser::{parse_toml};

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

    println!("config_file {} {}", config_file, Path::new(config_file).exists());
    println!("output_report_path {}", Path::new(output_report_path).exists());

    let config_path = Path::new(config_file);
    if (!config_path.exists()) {
        println!("config-file not found. (Value provided: {})", config_file);
        std::process::exit(1)
    }

    // parse_toml();


}






/// New
///     * Support running multiple config files
/// 
/// Read + Parse Config File
///     * Support Toml
///     * Support Dahl
///     * Check there is only one Setup and Teardown and there are Tests
/// Run Processes
///     * Run Setup, Tests, Teardown (CommandSet)
///     * Handle Response
///     * Write to Report JSON output
///     * Write to Console in readable format
/// Run Setup (Optional) (CommandSet)
///     * If setup fails, don't run Tests
///     * Setup is like a Test but if it fails then stop
/// Run Teardown (Optional) (CommandSet)
///     * Run all teardowns, even if one fails
///     * Teardown is like a Test
/// Run Tests
///     * Set of CommandSet (CommandFamily)
///     * Run CommandSet Serially
///     * Run CommandSet in Parallel
///         * Control the paralelism, default to n
/// Run Test
///     * CommandSet -> CommandSetResult 
///     * CommandSetResult: { CommandResult <Vec<CommandResult>, Vec<CommandError>>
///         * success :: () -> bool
///         * errors :: () -> Vec<CommandError>
///     * CommandResult :: { Command, StdOut, StdErr, ExitCode }


enum ProcessingKind {
    Sequential,
    Parallel,
}

struct CommandFamily {
    sets: Vec<CommandSet>,
    processing_kind: ProcessingKind,
}

enum CommandSetType {
    Setup,
    Test,
    Teardown,
}

struct CommandSet {
    set_type: CommandSetType,
    commands: Vec<ExecutableCommand>,
    processing_kind: ProcessingKind,
}

struct ExecutableCommand {
    name: Option<String>,
    description: Option<String>,
    cmd: String,
}

struct CommandResult {
    command: ExecutableCommand,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: u32,
}

struct CommandSetResult {
    results: Vec<CommandResult>
}
impl CommandSetResult {
    fn success(&self) -> bool {
        true
    }
    fn errors(&self) -> Vec<CommandResult> {
        //Filter all failures
        Vec::new()
    }
}

