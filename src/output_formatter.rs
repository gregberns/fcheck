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
    pub result: String,
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

fn result_to_string(b: bool) -> String {
    if b { "success".to_string() } else { "failure".to_string() }
}

fn map_module(module: &ProcessingModuleResult) -> ModuleOutput {
    ModuleOutput {
        result: result_to_string(module.success()),
        setup: module.setup.results.iter().map(map_command).collect(),
        tests: module.tests.sets.iter().map(map_test).collect(),
        teardown: module.teardown.results.iter().map(map_command).collect(),
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
    CommandOutput {
        name: res.command.name.clone(),
        result: result_to_string(res.success()),
        command: res.command.cmd.clone(),
        stdout: res.stdout.clone(),
        stderr: res.stderr.clone(),
        exit_code: res.exit_code.clone(),
        unknown_error: res.unknown_error.clone(),
    }
}

pub fn to_json(module: &ModuleOutput) -> String {
    serde_json::to_string_pretty(module)
      .expect("Failed to serialize string")
}
