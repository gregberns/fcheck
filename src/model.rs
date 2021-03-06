#[derive(Debug, PartialEq, Clone)]
pub struct Shell(pub String, pub Vec<String>);
impl Default for Shell {
    fn default() -> Shell {
        Shell("/bin/bash".to_string(), vec!["-c".to_string()])
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcessingModule {
    pub shell: Shell,
    pub setup: CommandSet,
    pub tests: CommandFamily,
    pub teardown: CommandSet,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessingKind {
    Serial,
    // Parallel,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandFamily {
    pub sets: Vec<CommandSet>,
    pub processing_kind: ProcessingKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandSetType {
    Setup,
    Test,
    Teardown,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandSet {
    pub name: Option<String>,
    pub set_type: CommandSetType,
    pub commands: Vec<ExecutableCommand>,
    pub processing_kind: ProcessingKind,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExecutableCommand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub timeout: Option<u64>,
    pub shell: Shell,
    pub cmd: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcessingModuleResult {
    pub module: ProcessingModule,
    pub setup: CommandSetResult,
    pub tests: Option<CommandFamilyResult>,
    pub teardown: Option<CommandSetResult>,
}
impl ProcessingModuleResult {
    pub fn success(&self) -> bool {
        self.setup.success()
            // Check that tests was Some and .success() is true
            && self.tests.is_some()
            && self.tests.clone().map(|t| t.success()).unwrap_or(false)
            // Check that teardown was Some and .success() is true
            && self.teardown.is_some()
            && self.teardown.clone().map(|t| t.success()).unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandFamilyResult {
    pub family: CommandFamily,
    pub sets: Vec<CommandSetResult>,
}
impl CommandFamilyResult {
    pub fn success(&self) -> bool {
        self.sets.iter().fold(true, |b, res| b && res.success())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandSetResult {
    pub set: CommandSet,
    pub results: Vec<CommandResult>,
}
impl CommandSetResult {
    pub fn success(&self) -> bool {
        self.results.iter().fold(true, |b, res| b && res.success())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommandResult {
    OsError {
        command: ExecutableCommand,
        error: String,
    },
    RuntimeError {
        command: ExecutableCommand,
        stdout: String,
        stderr: String,
        error: String,
    },
    Timeout {
        command: ExecutableCommand,
        stdout: String,
        stderr: String,
    },
    IrregularExitCode {
        command: ExecutableCommand,
        stdout: String,
        stderr: String,
        exit_code: String,
    },
    StandardResult {
        command: ExecutableCommand,
        stdout: String,
        stderr: String,
        exit_code: u32,
    },
}
impl CommandResult {
    pub fn success(&self) -> bool {
        match self {
            CommandResult::OsError {
                command: _,
                error: _,
            } => false,
            CommandResult::RuntimeError {
                command: _,
                stdout: _,
                stderr: _,
                error: _,
            } => false,
            CommandResult::Timeout {
                command: _,
                stdout: _,
                stderr: _,
            } => false,
            CommandResult::IrregularExitCode {
                command: _,
                stdout: _,
                stderr: _,
                exit_code: _,
            } => false,
            CommandResult::StandardResult {
                command: _,
                stdout: _,
                stderr: _,
                exit_code,
            } => exit_code.eq(&0),
        }
    }
}
