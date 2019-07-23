#[derive(Debug, PartialEq, Clone)]
pub struct ProcessingModule {
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
    pub cmd: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcessingModuleResult {
    pub module: ProcessingModule,
    pub setup: CommandSetResult,
    pub tests: CommandFamilyResult,
    pub teardown: CommandSetResult,
}
impl ProcessingModuleResult {
    pub fn success(&self) -> bool {
        self.setup.success() && self.tests.success() && self.teardown.success()
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
// pub struct CommandResult {
//     pub command: ExecutableCommand,
//     pub stdout: String,
//     pub stderr: String,
//     pub exit_code: String,
//     pub unknown_error: Option<String>,
// }
pub enum CommandResult {
    OsError {
        command: ExecutableCommand,
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
            CommandResult::OsError{command: _, error: _} => false,
            CommandResult::Timeout{command: _, stdout: _, stderr: _} => false,
            CommandResult::IrregularExitCode{command: _, stdout: _, stderr: _, exit_code: _} => false,
            CommandResult::StandardResult{command: _, stdout: _, stderr: _, exit_code} => exit_code.eq(&0),
        }
    }
}
