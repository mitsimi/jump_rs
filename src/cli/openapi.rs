use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::api;

#[derive(Subcommand)]
pub enum OpenApiCommands {
    /// Write the `OpenApi` document to stdout
    Emit,

    /// Write the `OpenApi` document to a file
    #[command(visible_alias = "gen")]
    Generate {
        /// Output file path
        #[arg(value_name = "PATH", default_value = "openapi.json")]
        path: PathBuf,
    },
}

impl OpenApiCommands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Emit => emit_openapi_spec(),
            Self::Generate { path } => generate_openapi_file(&path),
        }
    }
}

fn emit_openapi_spec() -> Result<()> {
    let json = api::openapi()
        .to_pretty_json()
        .context("failed to serialize the OpenAPI document")?;
    println!("{json}");
    Ok(())
}

fn generate_openapi_file(path: &PathBuf) -> Result<()> {
    let json = api::openapi()
        .to_pretty_json()
        .context("failed to serialize the OpenAPI document")?;
    std::fs::write(path, json)
        .with_context(|| format!("failed to write OpenAPI document to {}", path.display()))?;
    println!("Generated OpenAPI spec to: {}", path.display());
    Ok(())
}
