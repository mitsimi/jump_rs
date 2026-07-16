use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use std::io::{self, Write};

#[derive(Subcommand)]
pub enum UserCommands {
    /// Generate a bcrypt credential for auth.users
    #[command(arg_required_else_help = true)]
    Create(CreateUserArgs),
}

#[derive(Args)]
pub struct CreateUserArgs {
    /// Username to include in the generated credential
    #[arg(
        short = 'u',
        long,
        required_unless_present = "interactive",
        conflicts_with = "interactive"
    )]
    username: Option<String>,

    /// Password to hash (visible in shell history; prefer --interactive)
    #[arg(
        short = 'p',
        long,
        required_unless_present = "interactive",
        conflicts_with = "interactive"
    )]
    password: Option<String>,

    /// Prompt for a username and password without echoing the password
    #[arg(short = 'i', long, conflicts_with_all = ["username", "password"])]
    interactive: bool,

    /// Escape dollar signs for Docker Compose interpolation
    #[arg(long)]
    docker: bool,
}

impl UserCommands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Create(args) => {
                let (username, password) = if args.interactive {
                    interactive_credentials()?
                } else {
                    (
                        args.username
                            .context("--username is required unless --interactive is used")?,
                        args.password
                            .context("--password is required unless --interactive is used")?,
                    )
                };

                validate_username(&username)?;
                if password.is_empty() {
                    bail!("password cannot be empty");
                }

                let hash = bcrypt::hash(password, 12).context("failed to hash the password")?;
                println!("{}", format_credential(&username, &hash, args.docker));
                Ok(())
            }
        }
    }
}

fn interactive_credentials() -> Result<(String, String)> {
    let username = prompt("Username: ")?;
    let password = prompt_password("Password: ")?;
    let confirmation = prompt_password("Confirm password: ")?;

    if password != confirmation {
        bail!("passwords do not match");
    }

    Ok((username, password))
}

fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush().context("failed to write prompt")?;

    let mut value = String::new();
    io::stdin()
        .read_line(&mut value)
        .context("failed to read input")?;
    Ok(value.trim().to_owned())
}

fn prompt_password(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush().context("failed to write prompt")?;
    rpassword::read_password().context("failed to read password")
}

fn validate_username(username: &str) -> Result<()> {
    if username.trim().is_empty() {
        bail!("username cannot be empty");
    }
    if username != username.trim() {
        bail!("username cannot start or end with whitespace");
    }
    if username.contains(':') {
        bail!("username cannot contain ':'");
    }
    if username.contains(',') {
        bail!("username cannot contain ','");
    }
    Ok(())
}

fn format_credential(username: &str, hash: &str, docker: bool) -> String {
    let hash = if docker {
        hash.replace('$', "$$")
    } else {
        hash.to_owned()
    };
    format!("{username}:{hash}")
}

#[cfg(test)]
mod tests {
    use super::{format_credential, validate_username};

    #[test]
    fn accepts_a_valid_username() {
        assert!(validate_username("alice").is_ok());
    }

    #[test]
    fn rejects_usernames_that_break_the_auth_users_format() {
        for username in ["", " alice", "alice ", "alice:bob", "alice,bob"] {
            assert!(
                validate_username(username).is_err(),
                "{username:?} should be rejected"
            );
        }
    }

    #[test]
    fn docker_credentials_escape_dollar_signs() {
        assert_eq!(
            format_credential("alice", "$2b$12$hash", true),
            "alice:$$2b$$12$$hash"
        );
    }
}
