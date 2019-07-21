use std::str::{from_utf8};
use serde_json::*; //::to_string_pretty;
use serde_derive::Serialize;

use crate::model::{
    ExecutableCommand,
    CommandResult,
    CommandSetResult,
    CommandFamilyResult,
    ProcessingModuleResult,
    };

#[derive(Serialize, Debug)]
pub struct ModuleOutput {
    setup: Vec<CommandOutput>,
    tests: Vec<TestOutput>,
    teardown: Vec<CommandOutput>,
}

#[derive(Serialize, Debug)]
pub struct TestOutput {
    name: Option<String>,
    result: String,
    commands: Vec<CommandOutput>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CommandOutput {
    pub name: Option<String>,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: String,
    pub unknown_error: Option<String>,
}

pub fn format_module(module: &ProcessingModuleResult) -> String {
    let mod_out = map_module(module);
    to_json(&mod_out)
}

fn map_module(module: &ProcessingModuleResult) -> ModuleOutput {
    ModuleOutput {
        setup: module.setup.results.iter().map(map_command).collect(),
        tests: module.tests.sets.iter().map(map_test).collect(),
        teardown: module.teardown.results.iter().map(map_command).collect(),
    }
}

fn map_test(set: &CommandSetResult) -> TestOutput {
    TestOutput {
        name: set.set.name.clone(),
        result: if set.success() { "success".to_string() } else { "failure".to_string() },
        commands: set.results.iter().map(map_command).collect(),
    }
}

fn map_command(res: &CommandResult) -> CommandOutput {
    CommandOutput {
        name: res.command.name.clone(),
        command: res.command.cmd.clone(),
        stdout: from_utf8(&res.stdout).unwrap_or("Not UTF8 String").to_string(),
        stderr: from_utf8(&res.stderr).unwrap_or("Not UTF8 String").to_string(),
        exit_code: res.exit_code.clone(),
        unknown_error: res.unknown_error.clone(),
    }
}

pub fn to_json(module: &ModuleOutput) -> String {
    serde_json::to_string_pretty(module)
      .expect("Failed to serialize string")
}
