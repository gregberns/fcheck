#[derive(Debug, PartialEq)]
pub struct ProcessingModule {
    pub setup: CommandSet,
    pub tests: CommandFamily,
    pub teardown: CommandSet,
}

#[derive(Debug, PartialEq)]
pub enum ProcessingKind {
    Serial,
    Parallel,
}

#[derive(Debug, PartialEq)]
pub struct CommandFamily {
    pub sets: Vec<CommandSet>,
    pub processing_kind: ProcessingKind,
}

#[derive(Debug, PartialEq)]
pub enum CommandSetType {
    Setup,
    Test,
    Teardown,
}

#[derive(Debug, PartialEq)]
pub struct CommandSet {
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
pub struct CommandResult {
    pub command: ExecutableCommand,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: String,
    pub unknown_error: Option<String>,
}
impl CommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == "0".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandSetResult {
    pub results: Vec<CommandResult>
}
impl CommandSetResult {
    pub fn success(&self) -> bool {
        true
    }
    pub fn errors(&self) -> Vec<CommandResult> {
        //Filter all failures
        Vec::new()
    }
}
