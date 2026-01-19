//! Authentication commands with interactive prompts

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Password};

use crate::client::ApiClient;

/// Interactive login with encrypted password
pub async fn login_interactive(client: &ApiClient) -> Result<()> {
    println!("{}", "Instagram Login".bold().cyan());
    println!("{}", "━".repeat(40).dimmed());
    println!(
        "{}",
        "Your password will be encrypted before transmission.".dimmed()
    );
    println!();

    // Prompt for username
    let username: String = Input::new()
        .with_prompt("Username")
        .interact_text()?;

    // Prompt for password (hidden input)
    let password: String = Password::new()
        .with_prompt("Password")
        .interact()?;

    println!();
    println!("{}", "Authenticating...".dimmed());

    // Attempt login with encrypted password
    match client.login(&username, &password).await {
        Ok(response) => {
            if response.success {
                println!("{} {}", "✓".green().bold(), "Login successful!".green());
                if let Some(user) = response.user {
                    println!(
                        "  {} {} ({})",
                        "Logged in as:".dimmed(),
                        user.username.bold(),
                        user.full_name.unwrap_or_default()
                    );
                }
            } else {
                println!(
                    "{} {}",
                    "✗".red().bold(),
                    response.message.unwrap_or("Login failed".to_string()).red()
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("{} {}", "✗".red().bold(), format!("{}", e).red());
            Err(e)
        }
    }
}

/// Login with provided credentials (non-interactive)
pub async fn login_with_credentials(
    client: &ApiClient,
    username: &str,
    password: &str,
) -> Result<()> {
    println!("{}", "Authenticating...".dimmed());

    match client.login(username, password).await {
        Ok(response) => {
            if response.success {
                println!("{} {}", "✓".green().bold(), "Login successful!".green());
                if let Some(user) = response.user {
                    println!(
                        "  {} {} ({})",
                        "Logged in as:".dimmed(),
                        user.username.bold(),
                        user.full_name.unwrap_or_default()
                    );
                }
            } else {
                println!(
                    "{} {}",
                    "✗".red().bold(),
                    response.message.unwrap_or("Login failed".to_string()).red()
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("{} {}", "✗".red().bold(), format!("{}", e).red());
            Err(e)
        }
    }
}

/// Logout from Instagram
pub async fn logout(client: &ApiClient) -> Result<()> {
    println!("{}", "Logging out...".dimmed());

    client.logout().await?;
    println!("{} {}", "✓".green().bold(), "Logged out successfully".green());
    Ok(())
}

/// Check authentication status
pub async fn status(client: &ApiClient) -> Result<()> {
    match client.health().await {
        Ok(health) => {
            println!("{}", "Server Status".bold().cyan());
            println!("{}", "━".repeat(40).dimmed());
            println!(
                "  {} {}",
                "Server:".dimmed(),
                health.status.green()
            );
            if health.authenticated {
                println!(
                    "  {} {} ({})",
                    "Status:".dimmed(),
                    "Authenticated".green(),
                    health.username.unwrap_or_default().bold()
                );
            } else {
                println!(
                    "  {} {}",
                    "Status:".dimmed(),
                    "Not authenticated".yellow()
                );
            }
            Ok(())
        }
        Err(e) => {
            println!(
                "{} {} {}",
                "✗".red().bold(),
                "Cannot connect to server:".red(),
                e
            );
            Err(e)
        }
    }
}
