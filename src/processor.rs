use crate::model::{
    ProcessingModule, 
    ProcessingKind, 
    CommandFamily, 
    CommandSetType, 
    CommandSet,
    ExecutableCommand,
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

pub fn run(module: ProcessingModule) -> CommandSetResult {

}

pub fn run_commandset(stop_on_failure: bool, set: CommandSet) -> CommandSetResult {

}

pub fn run_commandfamily(family: CommandFamily) -> CommandFamilyResult {

}
