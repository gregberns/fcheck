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

#[derive(Debug, PartialEq)]
pub struct ExecutableCommand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cmd: String,
}

#[derive(Debug, PartialEq)]
pub struct CommandResult {
    pub command: ExecutableCommand,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: u32,
}

#[derive(Debug, PartialEq)]
pub struct CommandSetResult {
    pub results: Vec<CommandResult>
}
impl CommandSetResult {
    fn success(&self) -> bool {
        true
    }
    fn errors(&self) -> Vec<CommandResult> {
        //Filter all failures
        Vec::new()
    }
}
