//! Instagram DM CLI
//!
//! A command-line interface for Instagram Direct Messages.
//! Communicates with a local Python/FastAPI server that handles Instagram API.

mod client;
mod commands;
mod crypto;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use client::ApiClient;

/// Instagram DM CLI - Manage your Instagram DMs from the terminal
#[derive(Parser)]
#[command(name = "insta")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Server URL (default: http://localhost:8000)
    #[arg(short, long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to Instagram (interactive prompts for credentials)
    Login {
        /// Username (optional - will prompt if not provided)
        #[arg(short, long)]
        username: Option<String>,

        /// Password (optional - will prompt securely if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Logout from Instagram
    Logout,

    /// Check server status and authentication
    Status,

    /// Show inbox (list of conversations)
    Inbox {
        /// Number of threads to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Show messages in a thread
    Thread {
        /// Thread ID
        thread_id: String,

        /// Number of messages to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Send a message to a user by username
    Send {
        /// Username to send to (without @)
        username: String,

        /// Message text (optional - will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Reply to a thread
    Reply {
        /// Thread ID
        thread_id: String,

        /// Message text (optional - will prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Start interactive chat with a user
    Chat {
        /// Username to chat with (without @)
        username: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = ApiClient::new(cli.server.as_deref());

    match cli.command {
        Commands::Login { username, password } => {
            if let (Some(u), Some(p)) = (username.as_ref(), password.as_ref()) {
                // Non-interactive mode with provided credentials
                commands::login_with_credentials(&client, u, p).await
            } else if let Some(u) = username.as_ref() {
                // Username provided, prompt for password only
                use dialoguer::Password;
                println!("{}", "Instagram Login".bold().cyan());
                println!("{}", "━".repeat(40).dimmed());
                println!(
                    "{}",
                    "Your password will be encrypted before transmission.".dimmed()
                );
                println!();

                let password: String = Password::new()
                    .with_prompt("Password")
                    .interact()?;

                commands::login_with_credentials(&client, u, &password).await
            } else {
                // Full interactive mode
                commands::login_interactive(&client).await
            }
        }

        Commands::Logout => commands::logout(&client).await,

        Commands::Status => commands::status(&client).await,

        Commands::Inbox { limit } => commands::show_inbox(&client, limit).await,

        Commands::Thread { thread_id, limit } => {
            commands::show_thread(&client, &thread_id, limit).await
        }

        Commands::Send { username, message } => {
            commands::send_to_user(&client, &username, message.as_deref()).await
        }

        Commands::Reply { thread_id, message } => {
            commands::send_to_thread(&client, &thread_id, message.as_deref()).await
        }

        Commands::Chat { username } => commands::chat_with_user(&client, &username).await,
    }
}
