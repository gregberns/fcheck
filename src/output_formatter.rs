// use std::str::{from_utf8};
// use serde_json::*; //::to_string_pretty;
use serde_derive::Serialize;

use crate::model::{
    // ExecutableCommand,
    CommandResult,
    CommandSetResult,
    // CommandFamilyResult,
    ProcessingModuleResult,
};

#[derive(Serialize, Debug)]
pub struct ModuleOutput {
    result: String,
    setup: Vec<CommandOutput>,
    tests: Option<Vec<TestOutput>>,
    teardown: Option<Vec<CommandOutput>>,
}

#[derive(Serialize, Debug)]
pub struct TestOutput {
    name: Option<String>,
    result: String,
    commands: Vec<CommandOutput>,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CommandOutput {
    OsError {
        name: Option<String>,
        command: String,
        result: String,
        error: String,
    },
    RuntimeError {
        name: Option<String>,
        command: String,
        result: String,
        stdout: String,
        stderr: String,
        error: String,
    },
    Timeout {
        name: Option<String>,
        command: String,
        result: String,
        stdout: String,
        stderr: String,
    },
    IrregularExitCode {
        name: Option<String>,
        command: String,
        result: String,
        stdout: String,
        stderr: String,
        exit_code: String,
    },
    Complete {
        name: Option<String>,
        command: String,
        result: String,
        stdout: String,
        stderr: String,
        exit_code: u32,
    },
}

pub fn format_module(module: &ProcessingModuleResult) -> String {
    let mod_out = map_module(module);
    to_json(&mod_out)
}

fn result_to_string(b: bool) -> String {
    if b {
        "success".to_string()
    } else {
        "failure".to_string()
    }
}

fn map_module(module: &ProcessingModuleResult) -> ModuleOutput {
    ModuleOutput {
        result: result_to_string(module.success()),
        setup: module.setup.results.iter().map(map_command).collect(),
        tests: module
            .tests
            .clone()
            .map(|t| t.sets.iter().map(map_test).collect()),
        teardown: module
            .teardown
            .clone()
            .map(|t| t.results.iter().map(map_command).collect()),
    }
}

fn map_test(set: &CommandSetResult) -> TestOutput {
    TestOutput {
        name: set.set.name.clone(),
        result: result_to_string(set.success()),
        commands: set.results.iter().map(map_command).collect(),
    }
}

fn map_command(res: &CommandResult) -> CommandOutput {
    let result = result_to_string(res.success());

    match res {
        CommandResult::OsError { command, error } => CommandOutput::OsError {
            name: command.name.clone(),
            command: command.cmd.clone(),
            result: result,
            error: error.clone(),
        },
        CommandResult::RuntimeError {
            command,
            stdout,
            stderr,
            error,
        } => CommandOutput::RuntimeError {
            name: command.name.clone(),
            command: command.cmd.clone(),
            result: result,
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            error: error.clone(),
        },
        CommandResult::Timeout {
            command,
            stdout,
            stderr,
        } => CommandOutput::Timeout {
            name: command.name.clone(),
            command: command.cmd.clone(),
            result: result,
            stdout: stdout.clone(),
            stderr: stderr.clone(),
        },
        CommandResult::IrregularExitCode {
            command,
            stdout,
            stderr,
            exit_code,
        } => CommandOutput::IrregularExitCode {
            name: command.name.clone(),
            command: command.cmd.clone(),
            result: result,
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            exit_code: exit_code.clone(),
        },
        CommandResult::StandardResult {
            command,
            stdout,
            stderr,
            exit_code,
        } => CommandOutput::Complete {
            name: command.name.clone(),
            command: command.cmd.clone(),
            result: result,
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            exit_code: exit_code.clone(),
        },
    }
}

pub fn to_json(module: &ModuleOutput) -> String {
    serde_json::to_string_pretty(module).expect("Failed to serialize string")
}
