use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Result as IoResult;
use std::str::from_utf8;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use subprocess::{CaptureData, Exec, ExitStatus, Popen, PopenConfig, PopenError, Redirection};

use crate::model::{
    CommandFamily, CommandFamilyResult, CommandResult, CommandSet, CommandSetResult,
    CommandSetType, ExecutableCommand, ProcessingKind, ProcessingModule, ProcessingModuleResult,
    Shell,
};

// To Do
//     * If setup fails, don't run Tests
//     * Support Dahl

// New
//     * Support running multiple config files
//     * Run CommandSet in Parallel
//         * Control the paralelism, default to n
//     * (Optional) Write to Console in readable format (Not JSON)

pub fn run(module: &ProcessingModule) -> ProcessingModuleResult {
    run_processingmodule(&run_command, module)
}

pub fn run_processingmodule(
    run_cmd: &dyn Fn(&ExecutableCommand) -> CommandResult,
    module: &ProcessingModule,
) -> ProcessingModuleResult {
    let setup = run_commandset(true, &run_cmd, &module.setup);
    //Need ability to exit if there was a failure
    // StopOnSetupFailure = true && !setup.success()
    if setup.success() {
        let tests = run_commandfamily(&run_cmd, &module.tests);

        let teardown = run_commandset(false, &run_cmd, &module.teardown);

        ProcessingModuleResult {
            module: module.clone(),
            setup: setup,
            tests: Some(tests),
            teardown: Some(teardown),
        }
    } else {
        ProcessingModuleResult {
            module: module.clone(),
            setup: setup,
            tests: None,
            teardown: None,
        }
    }
}

pub fn run_commandfamily(
    run_cmd: &dyn Fn(&ExecutableCommand) -> CommandResult,
    family: &CommandFamily,
) -> CommandFamilyResult {
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
    run_cmd: &dyn Fn(&ExecutableCommand) -> CommandResult,
    set: &CommandSet,
) -> CommandSetResult {
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
    let timeout = command.timeout.map(|t| Duration::from_millis(t));
    let mut full_command = Vec::new();
    full_command.push(command.shell.0.clone());
    full_command.extend(command.shell.1.clone());
    full_command.push(command.cmd.clone());

    let res_data = start_process(timeout, &full_command);

    translate_result(&command, res_data)
}

struct CapturedData {
    stdout: Result<Vec<u8>, RunProcessError>,
    stderr: Result<Vec<u8>, RunProcessError>,
    exit_status: Result<Option<ExitStatus>, RunProcessError>,
}

#[derive(Debug)]
pub enum RunProcessError {
    // ProcessCreateError - PopenConfig::create error - If the external program cannot
    // be executed for any reason, an error is returned. The most typical reason for execution
    // to fail is that the program is missing on the PATH, but other errors are also possible.
    //  Note that this is distinct from the program running and then exiting with a failure
    // code - this can be detected by calling the wait method to obtain its exit status.
    ProcessCreateError(String),
    // Error from `wait_timeout` or `wait`
    ProcessRuntimeError(String),
    // RedirectReadFailed - `stderr.read_to_end` failed
    RedirectReadFailed(String),
    // Error when `thread.join()` called
    ThreadJoinError(String),
    // Fail to Kill process
    KillProcessError(String),
    KillWaitError(String),
}

fn start_process<S: AsRef<OsStr>>(
    timeout: Option<Duration>,
    args: &[S],
) -> Result<CapturedData, RunProcessError> {
    // let args_str = args.iter().fold(String::new(), |agg, i| {
    //     format!("{:?}, {:?}", agg, i.as_ref().to_os_string())
    // });
    // println!("Start Process = agrs: {:?}", args_str);

    let mut p = Popen::create(
        args,
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        },
    )
    .map_err(|err| RunProcessError::ProcessCreateError(err.to_string()))?;

    // This should only fail if `Redirection::Pipe` is not defined in `PopenConfig`....I think
    let (stdout, stderr) = (p.stdout.take().unwrap(), p.stderr.take().unwrap());

    fn spawn_thread(mut redirect: File) -> JoinHandle<IoResult<Vec<u8>>> {
        thread::spawn(move || {
            let mut buffer = Vec::new();
            redirect.read_to_end(&mut buffer)?;
            Ok(buffer)
        })
    }

    let out_handle: JoinHandle<IoResult<Vec<u8>>> = spawn_thread(stdout);
    let err_handle: JoinHandle<IoResult<Vec<u8>>> = spawn_thread(stderr);

    // both threads are now running _in parallel_
    let status: Result<Option<ExitStatus>, RunProcessError> = match timeout {
        Some(timeout) => p
            .wait_timeout(timeout)
            .map_err(|err| RunProcessError::ProcessRuntimeError(err.to_string())),
        None => p
            .wait()
            .map(|e| Some(e))
            .map_err(|err| RunProcessError::ProcessRuntimeError(err.to_string())),
    };

    fn collapse(
        result: std::result::Result<
            std::result::Result<std::vec::Vec<u8>, std::io::Error>,
            std::boxed::Box<dyn std::any::Any + std::marker::Send>,
        >,
    ) -> Result<Vec<u8>, RunProcessError> {
        match result {
            Ok(thread_result) => match thread_result {
                Ok(read_result) => Ok(read_result),
                Err(err) => Err(RunProcessError::RedirectReadFailed(err.to_string())),
            },
            Err(err) =>
            //trhead issue
            {
                Err(RunProcessError::ThreadJoinError(format!("{:?}", err)))
            }
        }
    }

    let out = collapse(out_handle.join());
    let err = collapse(err_handle.join());

    // `status == Ok(None)` means a timeout occured
    if let Ok(st) = status {
        if st.is_none() {
            //May not want to error out... As long as we can kill the process then were good.
            //Maybe log if the process can't be killed??
            p.kill()
                .map_err(|e| RunProcessError::KillProcessError(e.to_string()))?;
            p.wait()
                .map_err(|e| RunProcessError::KillWaitError(e.to_string()))?;
        }
    }

    // println!("Command Result: {:?}, {:?}, {:?}", out, err, status);

    Ok(CapturedData {
        stdout: out,
        stderr: err,
        exit_status: status,
    })
}

#[test]
fn test_sleep_with_timeout_fails() {
    let timeout = Duration::from_millis(100);
    let args = &vec!["sh", "-c", "echo hello && sleep 1"];

    let mut p = Popen::create(
        args,
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        },
    )
    .unwrap();

    let (mut stdout, mut stderr) = (p.stdout.take().unwrap(), p.stderr.take().unwrap());
    let out_handle: JoinHandle<IoResult<Vec<u8>>> = thread::spawn(move || {
        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer)?;
        Ok(buffer)
    });
    let err_handle: JoinHandle<IoResult<Vec<u8>>> = thread::spawn(move || {
        let mut buffer = Vec::new();
        stderr.read_to_end(&mut buffer)?;
        Ok(buffer)
    });
    // both threads are now running _in parallel_
    let status = p.wait_timeout(timeout).unwrap();
    let out = out_handle.join().unwrap();
    let err = err_handle.join().unwrap();

    assert!(status.is_none());

    if status.is_none() {
        p.kill().unwrap();
        p.wait().unwrap();
    }

    assert_eq!("hello\n", String::from_utf8(out.unwrap()).unwrap());
    assert_eq!("", String::from_utf8(err.unwrap()).unwrap());
}

fn translate_result(
    command: &ExecutableCommand,
    result: Result<CapturedData, RunProcessError>,
) -> CommandResult {
    fn translate_error(err: RunProcessError) -> String {
        match err {
            RunProcessError::ProcessCreateError(err) => format!("ProcessCreateError: {}", err),
            RunProcessError::ProcessRuntimeError(err) => format!("ProcessRuntimeError: {}", err),
            RunProcessError::RedirectReadFailed(err) => format!("RedirectReadFailed: {}", err),
            RunProcessError::ThreadJoinError(err) => format!("ThreadJoinError: {}", err),
            RunProcessError::KillProcessError(err) => format!("KillProcessError: {}", err),
            RunProcessError::KillWaitError(err) => format!("KillWaitError: {}", err),
        }
    }

    match result {
        Ok(res) => {
            let stdout = res.stdout.map_or_else(
                |e| format!("Fcheck error on stdout. {}", translate_error(e)),
                |v| from_utf8_lossy(v),
            );
            let stderr = res.stderr.map_or_else(
                |e| format!("Fcheck error on stderr. {}", translate_error(e)),
                |v| from_utf8_lossy(v),
            );
            match res.exit_status {
                Ok(opt_exit_status) => match opt_exit_status {
                    Some(exit_status) => match exit_status {
                        // https://docs.rs/subprocess/0.1.18/subprocess/enum.ExitStatus.html
                        ExitStatus::Exited(s) => CommandResult::StandardResult {
                            command: command.clone(),
                            stdout: stdout,
                            stderr: stderr,
                            exit_code: s.to_owned(),
                        },
                        ExitStatus::Signaled(s) => CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: stdout,
                            stderr: stderr,
                            exit_code: format!("Signaled({})", s),
                        },
                        ExitStatus::Other(s) => CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: stdout,
                            stderr: stderr,
                            exit_code: format!("Other({})", s),
                        },
                        ExitStatus::Undetermined => CommandResult::IrregularExitCode {
                            command: command.clone(),
                            stdout: stdout,
                            stderr: stderr,
                            exit_code: "Undetermined".to_string(),
                        },
                    },
                    None => {
                        //Timeout Occurred
                        CommandResult::Timeout {
                            command: command.clone(),
                            stdout: stdout,
                            stderr: stderr,
                        }
                    }
                },
                Err(err) => CommandResult::RuntimeError {
                    command: command.clone(),
                    stdout: stdout,
                    stderr: stderr,
                    error: format!("Runtime error occured: {}", translate_error(err)),
                },
            }
        }
        Err(e) => CommandResult::OsError {
            command: command.clone(),
            error: format!("OS error occured: {}", translate_error(e)),
        },
    }
}

fn from_utf8_lossy(vec_byte: Vec<u8>) -> String {
    String::from_utf8_lossy(&vec_byte).into_owned()
}

#[test]
fn t_exec_simple() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        timeout: None,
        shell: Shell::default(),
        cmd: "echo Hello".to_string(),
    };

    let res = run_command(&cmd);

    match res {
        CommandResult::StandardResult {
            command,
            stdout,
            stderr,
            exit_code,
        } => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "Hello\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        }
        _ => panic!("Fail"),
    }
}

#[test]
fn t_exec_multiline() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        timeout: None,
        shell: Shell::default(),
        cmd: r#"
            echo Hello;
            echo hello;
        "#
        .to_string(),
    };

    let res = run_command(&cmd);

    match res {
        CommandResult::StandardResult {
            command,
            stdout,
            stderr,
            exit_code,
        } => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "Hello\nhello\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        }
        _ => panic!("Fail"),
    }
}

#[test]
fn t_exec_runs_with_bash() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        timeout: None,
        shell: Shell::default(),
        cmd: r#"
            for (( i=0; i < 3; i++));
            do
                echo $i
            done;
        "#
        .to_string(),
    };

    let res = run_command(&cmd);
    match res {
        CommandResult::StandardResult {
            command,
            stdout,
            stderr,
            exit_code,
        } => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "0\n1\n2\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        }
        _ => panic!("Fail"),
    }
}

#[test]
fn t_execset_simple() {
    let cmds = CommandSet {
        name: None,
        set_type: CommandSetType::Test,
        commands: vec![
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                timeout: None,
                shell: Shell::default(),
                cmd: "echo Hello".to_string(),
            },
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                timeout: None,
                shell: Shell::default(),
                cmd: "echo Hello".to_string(),
            },
        ],
        processing_kind: ProcessingKind::Serial,
    };

    let res = run_commandset(true, &run_command, &cmds);

    assert_eq!(res.results.len(), 2);
}

#[test]
fn t_execset_stop_on_failure() {
    let cmds = CommandSet {
        name: None,
        set_type: CommandSetType::Test,
        commands: vec![
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                timeout: None,
                shell: Shell::default(),
                cmd: "exit 1".to_string(),
            },
            ExecutableCommand {
                name: Option::None,
                description: Option::None,
                timeout: None,
                shell: Shell::default(),
                cmd: "echo Hello".to_string(),
            },
        ],
        processing_kind: ProcessingKind::Serial,
    };

    let res = run_commandset(true, &run_command, &cmds);

    assert_eq!(res.results.len(), 1);
}
