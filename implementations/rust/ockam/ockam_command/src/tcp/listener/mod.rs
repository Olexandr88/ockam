mod create;
mod delete;
mod list;
mod show;

pub(crate) use create::CreateCommand;
pub(crate) use delete::DeleteCommand;
pub(crate) use list::ListCommand;
pub(crate) use show::ShowCommand;

use crate::CommandGlobalOpts;
use clap::{Args, Subcommand};

/// Manage TCP Listeners
#[derive(Args, Clone, Debug)]
pub struct TcpListenerCommand {
    #[command(subcommand)]
    subcommand: TcpListenerSubCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum TcpListenerSubCommand {
    /// Create tcp listener on the selected node
    Create(CreateCommand),

    /// Delete tcp listener on the selected node
    Delete(DeleteCommand),

    /// List tcp listeners registered on the selected node
    List(ListCommand),

    /// Show tcp listener details
    Show(ShowCommand),
}

impl TcpListenerCommand {
    pub fn run(self, options: CommandGlobalOpts) {
        match self.subcommand {
            TcpListenerSubCommand::Create(c) => c.run(options),
            TcpListenerSubCommand::Delete(c) => c.run(options),
            TcpListenerSubCommand::List(c) => c.run(options),
            TcpListenerSubCommand::Show(c) => c.run(options),
        }
    }

    pub fn name(&self) -> String {
        match &self.subcommand {
            TcpListenerSubCommand::Create(_) => "create tcp listener",
            TcpListenerSubCommand::Delete(_) => "delete tcp listener",
            TcpListenerSubCommand::List(_) => "list tcp listeners",
            TcpListenerSubCommand::Show(_) => "show tcp listener",
        }
        .to_string()
    }
}
