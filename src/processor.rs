use std::str::{from_utf8};
use subprocess::{Exec, CaptureData, ExitStatus, PopenError};

use crate::model::{
    ProcessingModule, 
    ProcessingKind, 
    CommandFamily, 
    CommandSetType, 
    CommandSet,
    ExecutableCommand,
    CommandResult,
    CommandSetResult,
    CommandFamilyResult,
    ProcessingModuleResult,
    };

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


pub fn run(module: &ProcessingModule) -> ProcessingModuleResult {
    run_processingmodule(&run_command, module)
}

pub fn run_processingmodule(
    run_cmd: &Fn(&ExecutableCommand) -> CommandResult,
    module: &ProcessingModule)
     -> ProcessingModuleResult {

    let setup = run_commandset(true, &run_command, &module.setup);

    let tests = run_commandfamily(&run_command, &module.tests);

    let teardown = run_commandset(false, &run_command, &module.teardown);

    ProcessingModuleResult {
        module: module.clone(),
        setup: setup,
        tests: tests,
        teardown: teardown,
    }
}

pub fn run_commandfamily(
    run_cmd: &Fn(&ExecutableCommand) -> CommandResult,
    family: &CommandFamily)
     -> CommandFamilyResult {

    let mut results = Vec::new();

    for set in family.sets.iter() {
        let res = run_commandset(true, run_cmd, &set);
        results.push(res);
    }
    
    CommandFamilyResult {
        family: family.clone(),
        sets: results,
    }
}

pub fn run_commandset(
    stop_on_failure: bool,
    run_cmd: &Fn(&ExecutableCommand) -> CommandResult,
    set: &CommandSet)
     -> CommandSetResult {

    let mut results = Vec::new();

    for cmd in set.commands.iter() {
        let res = run_cmd(&cmd);
        results.push(res.clone());
        if !res.success() && stop_on_failure {
            break;
        }
    }

    CommandSetResult {
        set: set.clone(),
        results: results,
    }
}

pub fn run_command(command: &ExecutableCommand) -> CommandResult {

    /// Support Timeout

    let res_data: Result<CaptureData, PopenError> = 
        Exec::shell(&command.cmd)
            // .popen()
            // .wait_timeout(5)
            .capture();

    /// This needs to be improved:
    /// * Return a valid exit_code
    ///     * If a bad one is returned, then include that in a 'uncommon' error
    /// * Return a result where errors are 'uncommon' errors

    match res_data {
        Ok(res) => {
            let exit_code = match res.exit_status {
                // https://docs.rs/subprocess/0.1.18/subprocess/enum.ExitStatus.html
                ExitStatus::Exited(i) => i.to_string(),
                ExitStatus::Signaled(i) => format!("Signaled({})", i),
                ExitStatus::Other(i) => format!("Other({})", i),
                ExitStatus::Undetermined => "Undetermined".to_string(),
            };

            CommandResult {
                command: command.clone(),
                stdout: res.stdout,
                stderr: res.stderr,
                exit_code: exit_code,
                unknown_error: Option::None,
            }
        },
        Err(e) => 
            CommandResult {
                command: command.clone(),
                stdout: vec!(),
                stderr: vec!(),
                exit_code: "-1".to_string(),
                unknown_error: Some(format!("Unknown error occured: {}", e)),
            }
    }
}

#[test]
fn t_exec_simple() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        cmd: "echo Hello".to_string(),
    };

    let res = run_command(&cmd);

    assert_eq!(res.command, cmd);
    assert_eq!(from_utf8(&res.stdout), Ok("Hello\n"));
    assert_eq!(res.stderr, Vec::new());
    assert_eq!(res.exit_code, "0".to_string());
    assert_eq!(res.unknown_error, Option::None);
}

#[test]
fn t_exec_multiline() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        cmd: r#"
            echo Hello;
            echo hello;
        "#.to_string(),
    };

    let res = run_command(&cmd);

    assert_eq!(res.command, cmd);
    assert_eq!(from_utf8(&res.stdout), Ok("Hello\nhello\n"));
    assert_eq!(res.stderr, Vec::new());
    assert_eq!(res.exit_code, "0".to_string());
    assert_eq!(res.unknown_error, Option::None);
}

#[test]
fn t_exec_runs_with_bash() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        cmd: r#"
            for (( i=0; i < 3; i++));
            do
                echo $i
            done;
        "#.to_string(),
    };

    let res = run_command(&cmd);

    assert_eq!(res.command, cmd);
    assert_eq!(from_utf8(&res.stdout), Ok("0\n1\n2\n"));
    assert_eq!(res.stderr, Vec::new());
    assert_eq!(res.exit_code, "0".to_string());
    assert_eq!(res.unknown_error, Option::None);
}


#[test]
fn t_execset_simple() {
    let cmds = CommandSet {
        set_type: CommandSetType::Test,
        commands: vec!(
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                cmd: "echo Hello".to_string(),
            },
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                cmd: "echo Hello".to_string(),
            },
        ),
        processing_kind: ProcessingKind::Serial,
    };

    let res = run_commandset(true, &run_command, &cmds);

    assert_eq!(res.results.len(), 2);
}

#[test]
fn t_execset_stop_on_failure() {
    let cmds = CommandSet {
        set_type: CommandSetType::Test,
        commands: vec!(
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                cmd: "exit 1".to_string(),
            },
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                cmd: "echo Hello".to_string(),
            },
        ),
        processing_kind: ProcessingKind::Serial,
    };

    let res = run_commandset(true, &run_command, &cmds);

    assert_eq!(res.results.len(), 1);
}
