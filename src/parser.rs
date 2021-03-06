use serde_derive::Deserialize;
use crate::model::{
    Shell,
    ProcessingModule, 
    ProcessingKind, 
    CommandFamily, 
    CommandSetType, 
    CommandSet,
    ExecutableCommand,
    };

pub enum FileType {
    Toml,
    Dhall,
}

#[derive(Deserialize, Debug)]
pub struct DefaultShell {
    path: String,
    args: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TestModule {
    version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    shell: Option<DefaultShell>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    setup: Option<Vec<Command>>,
    
    #[serde(alias = "test")]
    tests: Vec<Test>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    teardown: Option<Vec<Command>>,
}

#[derive(Deserialize, Debug)]
pub struct Test {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    
    #[serde(alias = "command")]
    commands: Vec<Command>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Command {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
    
    #[serde(alias = "cmd")]
    command: String,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
  description: String,
  line_col: Option<(usize, usize)>,
}

// Parse and return a ProcessingModule

pub fn prepare_file(
    file_type: FileType, 
    config_file: String) 
        -> Result<ProcessingModule, ParseError> {

    let module = match file_type {
        FileType::Toml => parse_toml(config_file)?,
        FileType::Dhall => panic!("Dhall not supported yet."),
    };

    Ok(testmodule_to_processingmodel(module))
}

pub fn file_extension_to_filetype(ext: &str) -> Option<FileType> {
    match ext {
        "toml" => Some(FileType::Toml),
        "dhall" => Some(FileType::Dhall),
        _ => Option::None,
    }
} 

// TOML Parser

fn parse_toml(config: String) -> Result<TestModule, ParseError> {
    toml::from_str(&config)
      .map_err(|e| {
        println!("HELLO {}", e);

        ParseError {
          description: e.to_string(),
          line_col: e.line_col(),
        }
      })
}

#[test]
fn t_basics() {
    let config = parse_toml(r#"
        version = "3"

        [shell]
        path = "/bin/bash"
        args = [
        "-c"
        ]

        [[setup]]
        name = "setup 1"
        cmd = "abc"
        [[setup]]
        command = "def"

        [[test]]
        name = "test 1"
        [[test.command]]
        name = "curl"
        timeout = 1000
        cmd = "curl google.com"
        [[test.command]]
        name = "ping"
        cmd = """
ping google.com;
curl google.com"""

        [[teardown]]
        name = "teardown 1"
        cmd = "abc"
        [[teardown]]
        command = "def"

    "#.to_string()).unwrap();

    assert_eq!(config.version, "3");

    assert!(config.shell.is_some());
    config.shell.map(|shell| {
        assert_eq!(shell.path, "/bin/bash");
        assert_eq!(shell.args, vec!("-c"));
    });
    
    config.setup.map(|a| {
        assert_eq!(a[0].name, Some("setup 1".to_string()));
        assert_eq!(a[0].command, "abc");
        assert_eq!(a[1].command, "def");
    });

    assert_eq!(config.tests[0].name, Some("test 1".to_string()));
    assert_eq!(config.tests[0].commands[0].name, Some("curl".to_string()));
    assert_eq!(config.tests[0].commands[0].timeout, Some(1000));
    assert_eq!(config.tests[0].commands[0].command, "curl google.com");
    assert_eq!(config.tests[0].commands[1].name, Some("ping".to_string()));
    assert_eq!(config.tests[0].commands[1].command, "ping google.com;\ncurl google.com");

    config.teardown.map(|a| {
        assert_eq!(a[0].name, Some("teardown 1".to_string()));
        assert_eq!(a[0].command, "abc");
        assert_eq!(a[1].command, "def");
    });
}

#[test]
fn t_setup_and_teardown_optional() {
    let config = parse_toml(r#"
        version = "3"

        [[test]]
        name = "test1"
        [[test.commands]]
        name = "curl"
        cmd = "curl google.com"

    "#.to_string()).unwrap();

    assert_eq!(config.setup, Option::None);
    assert_eq!(config.teardown, Option::None);
}

#[test]
fn t_parse_error() {
    let err: ParseError = parse_toml(r#"
        version = "3"

        garbage

    "#.to_string()).expect_err("Should have failed");

    assert_eq!(err, ParseError {
          description: "expected an equals, found a newline at line 4".to_string(),
          line_col: Some((3, 15)),
        });
}

// Maping from External API to Internal Model

fn testmodule_to_processingmodel(module: TestModule) -> ProcessingModule {
    let shell = match module.shell {
        Some(DefaultShell{path, args}) => Shell(path, args),
        None => Shell("/bin/bash".to_string(), vec!("-c".to_string())),
    };
    
    ProcessingModule {
        shell: shell.clone(),
        setup: commandlist_to_commandset(Some("Setup".to_string()), CommandSetType::Setup, &shell, module.setup),
        tests: testlist_to_commandfamily(&shell, module.tests),
        teardown: commandlist_to_commandset(Some("Teardown".to_string()), CommandSetType::Teardown, &shell, module.teardown),
    }
}

fn commandlist_to_commandset(name: Option<String>, c_type: CommandSetType, shell: &Shell, opt_commands: Option<Vec<Command>>) -> CommandSet {
    match opt_commands {
        Some(commands) => 
            CommandSet {
                name: name,
                set_type: c_type,
                commands: commands.iter().map(|c| command_to_execommand(shell, c)).collect(),
                processing_kind: ProcessingKind::Serial,
            },
        None => 
            CommandSet {
                name: name,
                set_type: c_type,
                commands: Vec::new(),
                processing_kind: ProcessingKind::Serial,
            },
    }
}

fn testlist_to_commandfamily(shell: &Shell, tests: Vec<Test>) -> CommandFamily {
    let command_sets = tests.iter()
        .map(|t| {
            CommandSet {
                name: t.name.clone(),
                set_type: CommandSetType::Test,
                commands: t.commands.iter().map(|c| command_to_execommand(shell, c)).collect(),
                processing_kind: ProcessingKind::Serial,
            }
        })
        .collect();
    
    CommandFamily {
        sets: command_sets,
        processing_kind: ProcessingKind::Serial,
    }
}

fn command_to_execommand(shell: &Shell, cmd: &Command) -> ExecutableCommand {
    ExecutableCommand {
        name: cmd.name.clone(),
        description: cmd.description.clone(),
        timeout: cmd.timeout.clone(),
        shell: shell.clone(),
        cmd: cmd.command.clone(),
    }
}

#[test]
fn t_map_module() {
    let res = testmodule_to_processingmodel(TestModule {
        version: "3".to_string(),
        shell: None,
        setup: Some(vec!(
            Command {
                name: Option::None,
                description: Option::None,
                timeout: None,
                command: "abc".to_string(),
            }
        )),
        tests: vec!(
            Test {
                name: Option::None,
                description: Option::None,
                commands: vec!(
                    Command {
                        name: Option::None,
                        description: Option::None,
                        timeout: None,
                        command: "def".to_string(),
                    }
                )
            }
        ),
        teardown: Some(vec!(
            Command {
                name: Option::None,
                description: Option::None,
                timeout: None,
                command: "ghi".to_string(),
            }
        )),
    });

    assert_eq!(res.setup.set_type, CommandSetType::Setup);
    assert_eq!(res.setup.commands[0].name, Option::None);
    assert_eq!(res.setup.commands[0].description, Option::None);
    assert_eq!(res.setup.commands[0].cmd, "abc");
    assert_eq!(res.setup.processing_kind, ProcessingKind::Serial);

    assert_eq!(res.tests.sets[0].set_type, CommandSetType::Test);
    assert_eq!(res.tests.sets[0].commands[0].name, Option::None);
    assert_eq!(res.tests.sets[0].commands[0].description, Option::None);
    assert_eq!(res.tests.sets[0].commands[0].cmd, "def");
    assert_eq!(res.tests.processing_kind, ProcessingKind::Serial);

    assert_eq!(res.teardown.set_type, CommandSetType::Teardown);
    assert_eq!(res.teardown.commands[0].name, Option::None);
    assert_eq!(res.teardown.commands[0].description, Option::None);
    assert_eq!(res.teardown.commands[0].cmd, "ghi");
    assert_eq!(res.teardown.processing_kind, ProcessingKind::Serial);
}
