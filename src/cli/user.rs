use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use dialoguer::{Confirm, Input, Password};

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
    #[arg(long, conflicts_with = "interactive")]
    docker: bool,
}

impl UserCommands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Create(args) => {
                let (username, password, docker) = if args.interactive {
                    interactive_credentials()?
                } else {
                    (
                        args.username
                            .context("--username is required unless --interactive is used")?,
                        args.password
                            .context("--password is required unless --interactive is used")?,
                        args.docker,
                    )
                };

                validate_username(&username)?;
                validate_password(&password)?;

                let hash = bcrypt::non_truncating_hash(password, bcrypt::DEFAULT_COST)
                    .context("failed to hash the password")?;
                println!("{}", format_credential(&username, &hash, docker));
                Ok(())
            }
        }
    }
}

fn interactive_credentials() -> Result<(String, String, bool)> {
    let username: String = Input::<String>::new()
        .with_prompt("Enter your username you want to use")
        .validate_with(|username: &String| validate_username(username))
        .interact_text()
        .context("failed to read username")?;
    let password = loop {
        let password = Password::new()
            .with_prompt("Set a password")
            .validate_with(|password: &String| validate_password(password))
            .interact()
            .context("failed to read password")?;
        let confirmation = Password::new()
            .with_prompt("Confirm password")
            .interact()
            .context("failed to read confirmation password")?;

        if password == confirmation {
            break password;
        }

        eprintln!("Passwords do not match. Please try again.");
    };

    let docker = Confirm::new()
        .with_prompt("Format for Docker Compose? (escapes dollar signs)")
        .default(false)
        .interact()
        .context("failed to read Docker Compose format preference")?;

    Ok((username, password, docker))
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

fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        bail!("password cannot be empty");
    }
    if password.len() > 72 {
        bail!("password cannot exceed bcrypt's 72-byte limit");
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
    use super::{format_credential, validate_password, validate_username};

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
    fn rejects_empty_or_too_long_passwords() {
        assert!(validate_password("").is_err());
        assert!(validate_password(&"a".repeat(72)).is_ok());
        assert!(validate_password(&"a".repeat(73)).is_err());
        assert!(validate_password(&"é".repeat(37)).is_err());
    }

    #[test]
    fn docker_credentials_escape_dollar_signs() {
        assert_eq!(
            format_credential("alice", "$2b$12$hash", true),
            "alice:$$2b$$12$$hash"
        );
    }
}
