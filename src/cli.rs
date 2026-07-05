use crate::api;
use clap::Parser;

#[derive(Parser)]
#[command(name = "jump.rs", version, about = "Wake-on-LAN web server")]
pub struct Cli {
    /// Print `OpenAPI` spec to stdout and exit
    #[arg(long)]
    emit_openapi: bool,

    /// Generate `OpenAPI` spec to file and exit
    #[arg(long)]
    gen_openapi: bool,
}

impl Cli {
    /// Handle CLI commands that should exit before running the server
    /// Returns true if a command was handled and the program should exit
    pub fn handle_commands(&self) -> bool {
        if self.emit_openapi {
            Self::emit_openapi_spec();
            return true;
        }

        if self.gen_openapi {
            Self::generate_openapi_file();
            return true;
        }

        false
    }

    fn emit_openapi_spec() {
        println!("{}", api::openapi().to_pretty_json().unwrap());
    }

    fn generate_openapi_file() {
        let spec = api::openapi();
        let json = serde_json::to_string_pretty(&spec).unwrap();
        std::fs::write("openapi.json", json).unwrap();
        println!("Generated OpenAPI spec to: openapi.json");
    }
}
