//! Send message commands

use anyhow::Result;
use dialoguer::Input;
use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};
use chrono::{Local, NaiveDateTime, TimeZone};
use tokio::sync::mpsc;

use crate::client::ApiClient;
use crate::colors::{Theme, instagram};
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

/// Interactive chat with a user by username (simple mode)
pub async fn chat_with_user(client: &ApiClient, username: &str) -> Result<()> {
    println!("{} {}", Theme::header("Chat with"), Theme::username(&format!("@{}", username)));
    println!(
        "{}",
        Theme::muted("Type your messages. Empty line to exit.")
    );
    println!();

    loop {
        let text: String = Input::new()
            .with_prompt(">")
            .allow_empty(true)
            .interact_text()?;

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

    Ok(())
}

/// Live chat mode with auto-polling for new messages
pub async fn live_chat_with_user(client: &ApiClient, username: &str) -> Result<()> {
    // First, find the thread ID for this user
    let spinner = create_spinner(&format!("Finding conversation with @{}", username));
    let inbox_response = client.get_inbox(50).await;
    spinner.finish_and_clear();

    let inbox_response = inbox_response?;

    let thread = inbox_response.threads.unwrap_or_default()
        .into_iter()
        .find(|t| t.users.iter().any(|u| u.username.eq_ignore_ascii_case(username)));

    let thread_id = match thread {
        Some(t) => t.id,
        None => {
            // No existing conversation, send a greeting to start one
            println!("{}", Theme::muted("No existing conversation. Send a message to start:"));
            let text: String = Input::new()
                .with_prompt(format!("Message to @{}", username))
                .interact_text()?;

            if text.trim().is_empty() {
                println!("{}", Theme::warning("Cannot start chat without a message."));
                return Ok(());
            }

            let spinner = create_spinner("Starting conversation...");
            let send_result = client.send_to_user(username, &text).await;
            spinner.finish_and_clear();

            match send_result {
                Ok(resp) if resp.success => {
                    println!("{} {}", Theme::check(), Theme::success("Message sent!"));
                    // Fetch inbox again to get thread ID
                    let inbox = client.get_inbox(50).await?;
                    inbox.threads.unwrap_or_default()
                        .into_iter()
                        .find(|t| t.users.iter().any(|u| u.username.eq_ignore_ascii_case(username)))
                        .map(|t| t.id)
                        .ok_or_else(|| anyhow::anyhow!("Could not find thread after sending message"))?
                }
                Ok(resp) => {
                    println!("{} {}", Theme::cross(), Theme::error(&resp.error.unwrap_or("Failed to send".to_string())));
                    return Ok(());
                }
                Err(e) => {
                    println!("{} {}", Theme::cross(), Theme::error(&format!("{}", e)));
                    return Err(e);
                }
            }
        }
    };

    // Start live chat UI
    run_live_chat(client, &thread_id, username).await
}

/// Events for the live chat
enum ChatEvent {
    NewMessages(Vec<DisplayMessage>),
    SendResult(bool),
}

/// Run the live chat interface with optimized rendering
async fn run_live_chat(client: &ApiClient, thread_id: &str, username: &str) -> Result<()> {
    let mut stdout = io::stdout();

    // Enter raw mode
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    // State
    let mut seen_messages: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut messages: Vec<DisplayMessage> = Vec::new();
    let mut input_buffer = String::new();
    let mut needs_redraw = true;
    let mut needs_input_redraw = false;

    // Initial fetch
    if let Ok(response) = client.get_thread(thread_id, 30).await {
        if let Some(thread) = response.thread {
            for msg in thread.messages.unwrap_or_default().into_iter().rev() {
                let sender = msg.user_id.as_ref()
                    .and_then(|uid| thread.users.iter().find(|u| &u.pk == uid))
                    .map(|u| u.username.clone())
                    .unwrap_or_else(|| "You".to_string());

                let is_me = sender == "You" || msg.user_id.is_none();
                let display_msg = DisplayMessage {
                    sender,
                    text: msg.text.unwrap_or_else(|| "[media]".to_string()),
                    timestamp: msg.timestamp.clone(),
                    is_me,
                };
                seen_messages.insert(msg.id);
                messages.push(display_msg);
            }
        }
    }

    // Channel for async events
    let (tx, mut rx) = mpsc::channel::<ChatEvent>(32);

    // Spawn polling task
    let poll_tx = tx.clone();
    let poll_thread_id = thread_id.to_string();
    let poll_client = client.clone();
    let poll_seen = seen_messages.clone();

    let poll_handle = tokio::spawn(async move {
        let mut seen = poll_seen;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            if let Ok(response) = poll_client.get_thread(&poll_thread_id, 30).await {
                if let Some(thread) = response.thread {
                    let mut new_msgs = Vec::new();
                    for msg in thread.messages.unwrap_or_default().into_iter().rev() {
                        if !seen.contains(&msg.id) {
                            let sender = msg.user_id.as_ref()
                                .and_then(|uid| thread.users.iter().find(|u| &u.pk == uid))
                                .map(|u| u.username.clone())
                                .unwrap_or_else(|| "You".to_string());

                            let is_me = sender == "You" || msg.user_id.is_none();
                            new_msgs.push(DisplayMessage {
                                sender,
                                text: msg.text.unwrap_or_else(|| "[media]".to_string()),
                                timestamp: msg.timestamp.clone(),
                                is_me,
                            });
                            seen.insert(msg.id);
                        }
                    }
                    if !new_msgs.is_empty() {
                        let _ = poll_tx.send(ChatEvent::NewMessages(new_msgs)).await;
                    }
                }
            }
        }
    });

    // Main loop - only handles input, minimal work
    loop {
        // Redraw only when needed
        if needs_redraw {
            draw_live_chat_full(&mut stdout, username, &messages, &input_buffer)?;
            needs_redraw = false;
        } else if needs_input_redraw {
            draw_input_line(&mut stdout, &input_buffer)?;
            needs_input_redraw = false;
        }

        // Check for async events (non-blocking)
        if let Ok(event) = rx.try_recv() {
            match event {
                ChatEvent::NewMessages(new_msgs) => {
                    for msg in new_msgs {
                        // Add to seen set via message text hash (since we don't have id here)
                        messages.push(msg);
                    }
                    needs_redraw = true;
                }
                ChatEvent::SendResult(_success) => {
                    // Could show send status
                }
            }
        }

        // Handle keyboard input (with short timeout for responsiveness)
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter => {
                            if !input_buffer.trim().is_empty() {
                                let text = input_buffer.clone();
                                input_buffer.clear();

                                // Add message to UI immediately
                                let display_msg = DisplayMessage {
                                    sender: "You".to_string(),
                                    text: text.clone(),
                                    timestamp: None,
                                    is_me: true,
                                };
                                messages.push(display_msg);
                                needs_redraw = true;

                                // Send in background
                                let send_client = client.clone();
                                let send_username = username.to_string();
                                let send_tx = tx.clone();
                                tokio::spawn(async move {
                                    let success = send_client.send_to_user(&send_username, &text).await
                                        .map(|r| r.success)
                                        .unwrap_or(false);
                                    let _ = send_tx.send(ChatEvent::SendResult(success)).await;
                                });
                            }
                        }
                        KeyCode::Backspace => {
                            if input_buffer.pop().is_some() {
                                needs_input_redraw = true;
                            }
                        }
                        KeyCode::Char(c) => {
                            input_buffer.push(c);
                            needs_input_redraw = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Cleanup
    poll_handle.abort();

    // Restore terminal
    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;

    println!("{}", Theme::muted("Exiting live chat."));
    Ok(())
}

/// Message for display
struct DisplayMessage {
    sender: String,
    text: String,
    timestamp: Option<String>,
    is_me: bool,
}

/// Draw the full live chat UI
fn draw_live_chat_full(
    stdout: &mut io::Stdout,
    username: &str,
    messages: &[DisplayMessage],
    input: &str,
) -> Result<()> {
    let (width, height) = terminal::size()?;
    let height = height as usize;
    let width = width as usize;

    execute!(stdout, cursor::MoveTo(0, 0), cursor::Hide)?;

    // Header (2 lines)
    let header = format!(" Live Chat with @{} ", username);
    let (r, g, b) = instagram::PINK;
    write!(stdout, "\x1b[48;2;{};{};{}m\x1b[38;2;255;255;255m{:^w$}\x1b[0m\r\n", r, g, b, header, w = width)?;

    let subheader = "ESC: exit • Auto-refresh: 3s";
    write!(stdout, "\x1b[38;2;142;142;142m{:^w$}\x1b[0m\r\n", subheader, w = width)?;

    // Messages area
    let msg_area_height = height.saturating_sub(4);
    let start_idx = messages.len().saturating_sub(msg_area_height);

    for i in 0..msg_area_height {
        execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;

        if let Some(msg) = messages.get(start_idx + i) {
            let time_str = msg.timestamp.as_ref()
                .map(|t| format_msg_time(t))
                .unwrap_or_default();

            // Clear line first
            write!(stdout, "\x1b[2K")?;

            if msg.is_me {
                // Right-aligned for sent messages
                let (r, g, b) = instagram::PURPLE;
                let content = if time_str.is_empty() {
                    msg.text.clone()
                } else {
                    format!("{} {}", msg.text, time_str)
                };
                let padding = width.saturating_sub(content.chars().count() + 1);
                write!(stdout, "\x1b[{}C\x1b[38;2;{};{};{}m{}\x1b[0m", padding, r, g, b, msg.text)?;
                if !time_str.is_empty() {
                    write!(stdout, " \x1b[38;2;142;142;142m{}\x1b[0m", time_str)?;
                }
            } else {
                // Left-aligned for received messages
                let (r, g, b) = instagram::PINK;
                write!(stdout, " \x1b[38;2;{};{};{}m{}\x1b[0m: {}", r, g, b, msg.sender, msg.text)?;
                if !time_str.is_empty() {
                    write!(stdout, " \x1b[38;2;142;142;142m{}\x1b[0m", time_str)?;
                }
            }
        } else {
            // Clear empty line
            write!(stdout, "\x1b[2K")?;
        }
    }

    // Input separator
    let input_y = (height - 2) as u16;
    execute!(stdout, cursor::MoveTo(0, input_y))?;
    let (r, g, b) = instagram::LIGHT_GRAY;
    write!(stdout, "\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, "─".repeat(width))?;

    // Input line
    execute!(stdout, cursor::MoveTo(0, input_y + 1))?;
    let (r, g, b) = instagram::ORANGE;
    write!(stdout, "\x1b[2K\x1b[38;2;{};{};{}m>\x1b[0m {}", r, g, b, input)?;

    // Show cursor at input position
    execute!(stdout, cursor::MoveTo((input.chars().count() + 2) as u16, input_y + 1), cursor::Show)?;

    stdout.flush()?;
    Ok(())
}

/// Draw only the input line (for fast typing feedback)
fn draw_input_line(stdout: &mut io::Stdout, input: &str) -> Result<()> {
    let (_width, height) = terminal::size()?;
    let input_y = (height - 1) as u16;

    execute!(stdout, cursor::MoveTo(0, input_y), cursor::Hide)?;

    let (r, g, b) = instagram::ORANGE;
    write!(stdout, "\x1b[2K\x1b[38;2;{};{};{}m>\x1b[0m {}", r, g, b, input)?;

    // Position cursor after input
    execute!(stdout, cursor::MoveTo((input.chars().count() + 2) as u16, input_y), cursor::Show)?;

    stdout.flush()?;
    Ok(())
}

/// Format message timestamp for display
fn format_msg_time(timestamp: &str) -> String {
    let naive = match NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => return String::new(),
    };

    let msg_time = match Local.from_local_datetime(&naive).single() {
        Some(dt) => dt,
        None => return String::new(),
    };

    msg_time.format("%H:%M").to_string()
}
