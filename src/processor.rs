use std::str::{from_utf8};
use std::time::Duration;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::io::Read;
use std::thread;
use std::thread::{JoinHandle};
use std::io::BufReader;

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
    let timeout = Duration::from_millis(500);

    println!("Start start_process");
    let res_data = start_process(timeout, &vec!("sh", "-c", &command.cmd));
    // let res_data = start_process(timeout, &vec!(&command.cmd));
    println!("End start_process");

    translate_result(&command, res_data)

    // This needs to be improved:
    // * Return a valid exit_code
    //     * If a bad one is returned, then include that in a 'uncommon' error
    // * Return a result where errors are 'uncommon' errors

}

struct CapturedData {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_status: Option<ExitStatus>,
}

fn start_process<S: AsRef<OsStr>>(timeout: Duration, args: &[S]) 
    -> Result<CapturedData, PopenError> {

    println!("Popen::create");
    let mut p = Popen::create(
        args, 
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,

            //pub executable: Option<OsString>,
            // executable: Some(OsString::from("/bin/bash".to_string())),
            // executable: Some(OsString::from("sh -c".to_string())),
            
            ..Default::default()
        })?;

    // println!("Start communicate_bytes");    
    // // -> IoResult<(Option<Vec<u8>>, Option<Vec<u8>>)>
    // let (maybe_out, maybe_err) = p.communicate_bytes(None)?;
    // println!("End communicate_bytes");
    
    // let out = maybe_out.unwrap_or_else(Vec::new);
    // let err = maybe_err.unwrap_or_else(Vec::new);
    
    
    // returns `Ok(None)` if the timeout is known to have elapsed.
    

    // These compile, but nothing is returned from each one
    // let out = p.stdout.as_ref().map(|mut f| {
    //         let mut buffer = Vec::new();
    //         f.read_to_end(&mut buffer).unwrap();
    //         buffer
    //     }).unwrap_or_else(|| Vec::new());
    // let err = p.stderr.as_ref().map(|mut f| {
    //         let mut buffer = Vec::new();
    //         f.read_to_end(&mut buffer).unwrap();
    //         buffer
    //     }).unwrap_or_else(|| Vec::new());

    // p.stdout

    // let thread_stdout = &p.stdout;
    // let mut thread_stderr = &p.stderr;
    
    let handle = thread::spawn(move || {
        println!("Start communicate_bytes");
        // -> IoResult<(Option<Vec<u8>>, Option<Vec<u8>>)>
        // let (maybe_out, maybe_err) = p.communicate_bytes(None).unwrap();

        // let out = p.stdout.map(|f| {
        //     let mut buffer = Vec::new();
            
        //     // let mut reader = BufReader::new(f);
        //     // let len = reader.read_to_end(&mut buffer).unwrap();

        //     // f.read_to_end(&mut buffer).unwrap();
        //     buffer = (&f).read_to_end().unwrap();
        //     buffer
        // }).unwrap_or_else(|| Vec::new());

        let out = p.stdout.as_ref().map(|mut f| {
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).unwrap();
            buffer
        }).unwrap_or_else(|| Vec::new());

        // let err = thread_stderr.as_ref().map(|mut f| {
        //     let mut buffer = Vec::new();
        //     f.read_to_end(&mut buffer).unwrap();
        //     buffer
        // }).unwrap_or_else(|| Vec::new());

        println!("End communicate_bytes");
        // (maybe_out, maybe_err)
        (out, Vec::new())
    });
    

    // let mut p = Exec::cmd("sleep").arg("2").popen()?;
    if let Some(status) = p.wait_timeout(Duration::new(1, 0))? {
        println!("process finished as {:?}", status);


        // let (maybe_out, maybe_err) = handle.join().unwrap();

        // let out = maybe_out.unwrap_or_else(Vec::new);
        // let err = maybe_err.unwrap_or_else(Vec::new);

        let (out, err) = handle.join().unwrap();

        Ok(CapturedData {
            stdout: out, stderr: err, exit_status: Some(status)
        })
    } else {
        // p.kill()?;
        // p.wait()?;

        let (out, err) = handle.join().unwrap();
        // let (maybe_out, maybe_err) = handle.join().unwrap();

        // let out = maybe_out.unwrap_or_else(Vec::new);
        // let err = maybe_err.unwrap_or_else(Vec::new);

        Ok(CapturedData {
            stdout: out, stderr: err, exit_status: None
        })


        // println!("process killed");
    }


    // let status = p.wait_timeout(timeout)?;
    // println!("End");


    // let out = match p.stdout.as_ref() {
    //     Some(mut f) => {
    //         let mut buffer = Vec::new();
    //         f.read_to_end(&mut buffer).unwrap();
    //         buffer
    //     },
    //     None => Vec::new()
    // };

    

    // let err = p.stderr.unwrap_or_else(Vec::new);



    // Ok(CapturedData {
    //     stdout: out, stderr: err, exit_status: status
    // })
}

// #[test]
// fn test_sleep_with_timeout_works() {
//     let timeout = Duration::from_millis(5);
//     let res_data = start_process(timeout, &vec!("sh", "-c", "sleep 5"));
// }

#[test]
fn test_sleep_with_timeout_fails() {
    
    let timeout = Duration::from_millis(5);
    let args = &vec!("sh", "-c", "sleep 5");

    let mut p = Popen::create(
        args, 
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Pipe,
            ..Default::default()
        }).unwrap();

    // let handle: JoinHandle<(Vec<u8>, Vec<u8>)> = thread::spawn(move || {
    //     // Can't figure out how to access `p.stdout` here
    //     let out = p.stdout.as_ref().map(|mut f| {
    //         let mut buffer = Vec::new();
    //         //And there are issues reading from mut file from immutible ref
    //         f.read_to_end(&mut buffer).unwrap();
    //         buffer
    //     }).unwrap_or_else(|| Vec::new());
    //     //stderr excluded...
    //     (out, Vec::new())
    // });

    let status = p.wait_timeout(timeout).unwrap();

    let (out, err) = handle.join().unwrap();

    //Other details here...
    //if status == None
    // p.kill()?;
    // p.wait()?;
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

fn from_utf8_lossy(vec_byte: Vec<u8>) -> String {
    String::from_utf8_lossy(&vec_byte).into_owned()
}

#[test]
fn t_exec_simple() {
    let cmd = ExecutableCommand {
        name: Option::None,
        description: Option::None,
        cmd: "echo Hello".to_string(),
    };

    let res = run_command(&cmd);

    match res {
        CommandResult::StandardResult {command, stdout, stderr, exit_code} => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "Hello\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        },
        _ => panic!("Fail")
    }
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

    match res {
        CommandResult::StandardResult {command, stdout, stderr, exit_code} => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "Hello\nhello\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        },
        _ => panic!("Fail")
    }
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
    match res {
        CommandResult::StandardResult {command, stdout, stderr, exit_code} => {
            assert_eq!(command, cmd);
            assert_eq!(stdout, "0\n1\n2\n");
            assert_eq!(stderr, "");
            assert_eq!(exit_code, 0);
        },
        _ => panic!("Fail")
    }
}


#[test]
fn t_execset_simple() {
    let cmds = CommandSet {
        name: None,
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
        name: None,
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
