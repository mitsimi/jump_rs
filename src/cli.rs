mod openapi;
mod user;

use crate::cli::{openapi::OpenApiCommands, user::UserCommands};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jumpers")]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a bcrypt credential for an authentication user
    User {
        #[command(subcommand)]
        command: UserCommands,
    },

    /// Emit or write the `OpenAPI` specification
    #[command(name = "openapi")]
    OpenApi {
        #[command(subcommand)]
        command: OpenApiCommands,
    },
}

impl Commands {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::User { command } => command.run(),
            Self::OpenApi { command } => command.run(),
        }
    }
}
