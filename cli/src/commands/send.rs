//! Send message commands

use anyhow::Result;
use colored::Colorize;
use dialoguer::Input;

use crate::client::ApiClient;

/// Send a message to a user (interactive or with provided message)
pub async fn send_to_user(client: &ApiClient, username: &str, message: Option<&str>) -> Result<()> {
    let text = match message {
        Some(m) => m.to_string(),
        None => {
            // Interactive mode
            Input::new()
                .with_prompt(format!("Message to @{}", username))
                .interact_text()?
        }
    };

    if text.trim().is_empty() {
        println!("{}", "Message cannot be empty.".yellow());
        return Ok(());
    }

    println!("{}", format!("Sending to @{}...", username).dimmed());

    match client.send_to_user(username, &text).await {
        Ok(response) => {
            if response.success {
                println!(
                    "{} {}",
                    "✓".green().bold(),
                    format!("Message sent to @{}", username).green()
                );
            } else {
                println!(
                    "{} {}",
                    "✗".red().bold(),
                    response.error.unwrap_or("Failed to send message".to_string()).red()
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

/// Send a message to an existing thread (interactive or with provided message)
pub async fn send_to_thread(
    client: &ApiClient,
    thread_id: &str,
    message: Option<&str>,
) -> Result<()> {
    let text = match message {
        Some(m) => m.to_string(),
        None => {
            // Interactive mode
            Input::new()
                .with_prompt("Message")
                .interact_text()?
        }
    };

    if text.trim().is_empty() {
        println!("{}", "Message cannot be empty.".yellow());
        return Ok(());
    }

    println!("{}", "Sending message...".dimmed());

    match client.send_to_thread(thread_id, &text).await {
        Ok(response) => {
            if response.success {
                println!("{} {}", "✓".green().bold(), "Message sent!".green());
            } else {
                println!(
                    "{} {}",
                    "✗".red().bold(),
                    response.error.unwrap_or("Failed to send message".to_string()).red()
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

/// Interactive chat with a user by username
pub async fn chat_with_user(client: &ApiClient, username: &str) -> Result<()> {
    println!("{} {}", "Chat with".bold().cyan(), format!("@{}", username).bold());
    println!(
        "{}",
        "Type your messages. Empty line to exit.".dimmed()
    );
    println!();

    loop {
        let text: String = Input::new()
            .with_prompt(">")
            .allow_empty(true)
            .interact_text()?;

        if text.trim().is_empty() {
            println!("{}", "Exiting chat mode.".dimmed());
            break;
        }

        match client.send_to_user(username, &text).await {
            Ok(response) => {
                if response.success {
                    println!("{}", "✓ Sent".green().dimmed());
                } else {
                    println!(
                        "{} {}",
                        "✗".red(),
                        response.error.unwrap_or("Failed".to_string()).red()
                    );
                }
            }
            Err(e) => {
                println!("{} {}", "✗".red(), format!("{}", e).red());
            }
        }
    }

    Ok(())
}
