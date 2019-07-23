// use std::str::{from_utf8};
use std::time::Duration;
use std::ffi::{OsStr, OsString};
use subprocess::{
    Exec, 
    CaptureData, 
    ExitStatus, 
    Popen,
    PopenConfig,
    PopenError,
    Redirection,
    };

use crate::model::{
    ProcessingModule, 
    // ProcessingKind, 
    CommandFamily, 
    // CommandSetType, 
    CommandSet,
    ExecutableCommand,
    CommandResult,
    CommandSetResult,
    CommandFamilyResult,
    ProcessingModuleResult,
    };

// New
//     * Support running multiple config files
// 
// Read + Parse Config File
//     * Support Toml
//     * Support Dahl
//     * Check there is only one Setup and Teardown and there are Tests
// Run Processes
//     * Run Setup, Tests, Teardown (CommandSet)
//     * Handle Response
//     * Write to Report JSON output
//     * Write to Console in readable format
// Run Setup (Optional) (CommandSet)
//     * If setup fails, don't run Tests
//     * Setup is like a Test but if it fails then stop
// Run Teardown (Optional) (CommandSet)
//     * Run all teardowns, even if one fails
//     * Teardown is like a Test
// Run Tests
//     * Set of CommandSet (CommandFamily)
//     * Run CommandSet Serially
//     * Run CommandSet in Parallel
//         * Control the paralelism, default to n
// Run Test
//     * CommandSet -> CommandSetResult 
//     * CommandSetResult: { CommandResult <Vec<CommandResult>, Vec<CommandError>>
//         * success :: () -> bool
//         * errors :: () -> Vec<CommandError>
//     * CommandResult :: { Command, StdOut, StdErr, ExitCode }


pub fn run(module: &ProcessingModule) -> ProcessingModuleResult {
    run_processingmodule(&run_command, module)
}

pub fn run_processingmodule(
    run_cmd: &Fn(&ExecutableCommand) -> CommandResult,
    module: &ProcessingModule)
     -> ProcessingModuleResult {

    let setup = run_commandset(true, &run_cmd, &module.setup);
    //Need ability to exit if there was a failure
    // StopOnSetupFailure = true && !setup.success()
    // if !setup.success() {
    //     ProcessingModuleResult {
    //         module: module.clone(),
    //         setup: setup,
    //         tests: None,
    //         teardown: None,
    //     }
    // }

    let tests = run_commandfamily(&run_cmd, &module.tests);

    let teardown = run_commandset(false, &run_cmd, &module.teardown);

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

    // Support Timeout

    //This works, but doesn't support timeout or switching which shell to use
    // let res_data: Result<CaptureData, PopenError> = 
    //     Exec::shell(&command.cmd)
    //         .capture();

    // let timeout = Duration::from_millis(command.timeout);
    let timeout = Duration::from_millis(0);

    let res_data = start_process(timeout, &vec!("sh", "-c", &command.cmd));

    translate_result(&command, res_data)

    // This needs to be improved:
    // * Return a valid exit_code
    //     * If a bad one is returned, then include that in a 'uncommon' error
    // * Return a result where errors are 'uncommon' errors

    // match res_data {
    //     Ok(res) => {
    //         let exit_code = match res.exit_status {
    //             // https://docs.rs/subprocess/0.1.18/subprocess/enum.ExitStatus.html
    //             ExitStatus::Exited(i) => i.to_string(),
    //             ExitStatus::Signaled(i) => format!("Signaled({})", i),
    //             ExitStatus::Other(i) => format!("Other({})", i),
    //             ExitStatus::Undetermined => "Undetermined".to_string(),
    //         };

    //         CommandResult {
    //             command: command.clone(),
    //             stdout: res.stdout_str(),
    //             stderr: res.stderr_str(),
    //             exit_code: exit_code,
    //             unknown_error: Option::None,
    //         }
    //     },
    //     Err(e) => 
    //         CommandResult {
    //             command: command.clone(),
    //             stdout: "".to_string(),
    //             stderr: "".to_string(),
    //             exit_code: "-1".to_string(),
    //             unknown_error: Some(format!("Unknown error occured: {}", e)),
    //         }
    // }
}

struct CapturedData {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_status: Option<ExitStatus>,
}

// pub struct CommandResult {
//     pub command: ExecutableCommand,
//     pub stdout: String,
//     pub stderr: String,
//     pub exit_code: String,
//     pub unknown_error: Option<String>,
// }

fn from_utf8_lossy(vec_byte: Vec<u8>) -> String {
    String::from_utf8_lossy(&vec_byte).into_owned()
}

fn translate_result(
    command: &ExecutableCommand,
    result: Result<CapturedData, PopenError>)
    -> CommandResult {
    match result {
        Ok(res) => {
            match res.exit_status {
                Some(exit_status) => match exit_status {
                    // https://docs.rs/subprocess/0.1.18/subprocess/enum.ExitStatus.html
                    ExitStatus::Exited(s) => {
                        CommandResult::StandardResult {
                            command: command.clone(),
                            stdout: from_utf8_lossy(res.stdout),
                            stderr: from_utf8_lossy(res.stderr),
                            exit_code: s.to_owned(),
                        }
                    },
                    ExitStatus::Signaled(s) =>
                        CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: from_utf8_lossy(res.stdout),
                            stderr: from_utf8_lossy(res.stderr),
                            exit_code: format!("Signaled({})", s),
                        },
                    ExitStatus::Other(s) =>
                        CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: from_utf8_lossy(res.stdout),
                            stderr: from_utf8_lossy(res.stderr),
                            exit_code: format!("Other({})", s),
                        },
                    ExitStatus::Undetermined => 
                        CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: from_utf8_lossy(res.stdout),
                            stderr: from_utf8_lossy(res.stderr),
                            exit_code: "Undetermined".to_string(),
                        },
                },
                None => {
                    //Timeout Occurred
                    CommandResult::Timeout {
                        command: command.clone(),
                        stdout: from_utf8_lossy(res.stdout),
                        stderr: from_utf8_lossy(res.stderr),
                    }
                },
            }
        },
        Err(e) => 
            CommandResult::OsError {
                command: command.clone(),
                error: format!("OS error occured: {}", e),
            }
    }
}

fn start_process<S: AsRef<OsStr>>(timeout: Duration, args: &[S]) 
    -> Result<CapturedData, PopenError> {

    let mut p = Popen::create(
        args, 
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        })?;
    // -> IoResult<(Option<Vec<u8>>, Option<Vec<u8>>)>
    let (maybe_out, maybe_err) = p.communicate_bytes(None)?;
    let out = maybe_out.unwrap_or_else(Vec::new);
    let err = maybe_err.unwrap_or_else(Vec::new);
    
    // returns `Ok(None)` if the timeout is known to have elapsed.
    let status = p.wait_timeout(timeout)?;


    Ok(CapturedData {
        stdout: out, stderr: err, exit_status: status
    })
}

// #[test]
// fn wait_timeout() {

//     start_process(Duration::from_millis(100), &["sleep", "0.5"]);



//     let mut p = Popen::create(&["sleep", "0.5"], 
//         PopenConfig {
//             stdout: Redirection::Pipe,
//             ..Default::default()
//         })
//         .unwrap();

//     p.wait_timeout(Duration::from_millis(100)).unwrap()

// // let mut p = Popen::create(&["sh", "-c", r#"test "$SOMEVAR" = "bar""#],
// //                               PopenConfig {
// //                                   stdout: Redirection::Pipe,
// //                                   env: Some(dups),
// //                                   ..Default::default()
// //                               }).unwrap();
// //     assert!(p.wait().unwrap().success());



//     let ret = p.wait_timeout(Duration::from_millis(100)).unwrap();
//     assert!(ret.is_none());
//     let ret = p.wait_timeout(Duration::from_millis(450)).unwrap();
//     assert_eq!(ret, Some(ExitStatus::Exited(0)));
// }

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
