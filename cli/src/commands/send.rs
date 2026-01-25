//! Send message commands

use anyhow::Result;
use dialoguer::Input;
use rustyline::error::ReadlineError;

use crate::client::ApiClient;
use crate::colors::Theme;
use crate::completer::create_editor;
use crate::spinner::create_spinner;

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
        println!("{}", Theme::warning("Message cannot be empty."));
        return Ok(());
    }

    let spinner = create_spinner(&format!("Sending to @{}...", username));

    let result = client.send_to_user(username, &text).await;
    spinner.finish_and_clear();

    match result {
        Ok(response) => {
            if response.success {
                println!(
                    "{} {}",
                    Theme::check(),
                    Theme::success(&format!("Message sent to @{}", username))
                );
            } else {
                println!(
                    "{} {}",
                    Theme::cross(),
                    Theme::error(&response.error.unwrap_or("Failed to send message".to_string()))
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("{} {}", Theme::cross(), Theme::error(&format!("{}", e)));
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
        println!("{}", Theme::warning("Message cannot be empty."));
        return Ok(());
    }

    let spinner = create_spinner("Sending message...");

    let result = client.send_to_thread(thread_id, &text).await;
    spinner.finish_and_clear();

    match result {
        Ok(response) => {
            if response.success {
                println!("{} {}", Theme::check(), Theme::success("Message sent!"));
            } else {
                println!(
                    "{} {}",
                    Theme::cross(),
                    Theme::error(&response.error.unwrap_or("Failed to send message".to_string()))
                );
            }
            Ok(())
        }
        Err(e) => {
            println!("{} {}", Theme::cross(), Theme::error(&format!("{}", e)));
            Err(e)
        }
    }
}

/// Interactive chat with a user by username (with tab completion for @mentions)
pub async fn chat_with_user(client: &ApiClient, username: &str, usernames: Vec<String>) -> Result<()> {
    println!("{} {}", Theme::header("Chat with"), Theme::username(&format!("@{}", username)));
    println!(
        "{}",
        Theme::muted("Type your messages. Press Tab to autocomplete @usernames. Empty line to exit.")
    );
    println!();

    let mut editor = create_editor(usernames);
    let prompt = format!("{} ", Theme::prompt(">"));

    loop {
        match editor.readline(&prompt) {
            Ok(text) => {
                if text.trim().is_empty() {
                    println!("{}", Theme::muted("Exiting chat mode."));
                    break;
                }

                let spinner = create_spinner("Sending...");
                let result = client.send_to_user(username, &text).await;
                spinner.finish_and_clear();

                match result {
                    Ok(response) => {
                        if response.success {
                            println!("{} {}", Theme::check(), Theme::muted("Sent"));
                        } else {
                            println!(
                                "{} {}",
                                Theme::cross(),
                                Theme::error(&response.error.unwrap_or("Failed".to_string()))
                            );
                        }
                    }
                    Err(e) => {
                        println!("{} {}", Theme::cross(), Theme::error(&format!("{}", e)));
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", Theme::muted("Interrupted. Exiting chat mode."));
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", Theme::muted("Exiting chat mode."));
                break;
            }
            Err(err) => {
                println!("{} {}", Theme::cross(), Theme::error(&format!("{}", err)));
                break;
            }
        }
    }

    Ok(())
}
