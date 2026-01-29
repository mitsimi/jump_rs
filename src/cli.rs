use crate::api;
use clap::Parser;
use utoipa::OpenApi;

#[derive(Parser)]
#[command(name = "jump.rs", version, about = "Wake-on-LAN web server")]
pub struct Cli {
    /// Print OpenAPI spec to stdout and exit
    #[arg(long)]
    emit_openapi: bool,

    /// Generate OpenAPI spec to file and exit
    #[arg(long)]
    gen_openapi: bool,
}

const FRONTEND_DIR: &str = "./frontend";

impl Cli {
    /// Handle CLI commands that should exit before running the server
    /// Returns true if a command was handled and the program should exit
    pub fn handle_commands(&self) -> bool {
        if self.emit_openapi {
            self.emit_openapi_spec();
            return true;
        }

        if self.gen_openapi {
            self.generate_openapi_file();
            return true;
        }

        false
    }

    fn emit_openapi_spec(&self) {
        println!("{}", api::ApiDoc::openapi().to_pretty_json().unwrap());
    }

    fn generate_openapi_file(&self) {
        let spec = api::ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec).unwrap();

        // Determine output directory based on current working directory
        let output_dir = std::env::current_dir()
            .ok()
            .and_then(|cwd| {
                cwd.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .map(|dir_name| {
                if dir_name == "frontend" {
                    ".".to_string()
                } else {
                    FRONTEND_DIR.to_string()
                }
            })
            .unwrap_or_else(|| FRONTEND_DIR.to_string());

        let output_path = format!("{}/openapi.json", output_dir);
        std::fs::write(&output_path, json).unwrap();
        println!("Generated OpenAPI spec to: {}", output_path);
    }
}
