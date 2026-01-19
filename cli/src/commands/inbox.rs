//! Inbox and thread commands

use anyhow::Result;
use colored::Colorize;

use crate::client::ApiClient;
use crate::models::Thread;

/// Display inbox (list of conversations)
pub async fn show_inbox(client: &ApiClient, limit: u32) -> Result<()> {
    println!("{}", "Fetching inbox...".dimmed());

    let response = client.get_inbox(limit).await?;

    if !response.success {
        println!(
            "{} {}",
            "✗".red().bold(),
            response.error.unwrap_or("Failed to fetch inbox".to_string()).red()
        );
        return Ok(());
    }

    let threads = response.threads.unwrap_or_default();

    if threads.is_empty() {
        println!("{}", "No conversations found.".dimmed());
        return Ok(());
    }

    println!();
    println!("{}", "Inbox".bold().cyan());
    println!("{}", "━".repeat(60).dimmed());

    for (i, thread) in threads.iter().enumerate() {
        print_thread_summary(i + 1, thread);
    }

    println!("{}", "━".repeat(60).dimmed());
    println!(
        "{}",
        format!("Showing {} conversations", threads.len()).dimmed()
    );

    Ok(())
}

/// Display a specific thread with messages
pub async fn show_thread(client: &ApiClient, thread_id: &str, limit: u32) -> Result<()> {
    println!("{}", "Fetching messages...".dimmed());

    let response = client.get_thread(thread_id, limit).await?;

    if !response.success {
        println!(
            "{} {}",
            "✗".red().bold(),
            response.error.unwrap_or("Failed to fetch thread".to_string()).red()
        );
        return Ok(());
    }

    let thread = match response.thread {
        Some(t) => t,
        None => {
            println!("{}", "Thread not found.".dimmed());
            return Ok(());
        }
    };

    println!();
    let participants: Vec<&str> = thread.users.iter().map(|u| u.username.as_str()).collect();
    println!(
        "{} {}",
        "Conversation with:".bold().cyan(),
        participants.join(", ").bold()
    );
    println!("{}", "━".repeat(60).dimmed());

    let messages = thread.messages.unwrap_or_default();

    if messages.is_empty() {
        println!("{}", "No messages in this thread.".dimmed());
        return Ok(());
    }

    for msg in messages.iter().rev() {
        // Find the sender
        let sender = msg.user_id.as_ref().and_then(|uid| {
            thread.users.iter().find(|u| &u.pk == uid)
        }).map(|u| u.username.as_str()).unwrap_or("You");

        let text = msg.text.as_deref().unwrap_or("[media]");
        let time = msg.timestamp.as_ref()
            .map(|t| format_time_ago(t))
            .unwrap_or_default();

        println!(
            "{} {} {}",
            sender.bold().blue(),
            time.dimmed(),
            ""
        );
        println!("  {}", text);
        println!();
    }

    println!("{}", "━".repeat(60).dimmed());
    println!(
        "{}",
        format!("Thread ID: {}", thread_id).dimmed()
    );

    Ok(())
}

/// Print a thread summary for inbox view
fn print_thread_summary(index: usize, thread: &Thread) {
    // Get username for sending messages
    let username = thread.users.first().map(|u| u.username.as_str()).unwrap_or("unknown");

    // Use thread_title if available, otherwise use username
    let title = thread
        .thread_title
        .clone()
        .unwrap_or_else(|| username.to_string());

    let preview = thread
        .last_message_text
        .clone()
        .unwrap_or_else(|| "[media]".to_string());

    // Truncate preview
    let preview = if preview.chars().count() > 35 {
        format!("{}...", preview.chars().take(35).collect::<String>())
    } else {
        preview
    };

    // Unread indicator
    let unread = if thread.has_unread.unwrap_or(false) { "●".blue() } else { " ".normal() };

    // Time
    let time = thread
        .last_message_timestamp
        .as_ref()
        .map(|t| format_time_ago(t))
        .unwrap_or_default();

    // Show: "1. Display Name (@username) 13d"
    println!(
        "{:>3}. {} {} {} {}",
        index.to_string().dimmed(),
        title.bold(),
        format!("@{}", username).cyan(),
        time.dimmed(),
        unread
    );
    println!("     {} {}", "└".dimmed(), preview);
}

/// Format ISO timestamp to relative time
fn format_time_ago(timestamp: &str) -> String {
    // Parse "2026-01-14T12:33:38" format
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Simple parsing - extract date parts
    let parts: Vec<&str> = timestamp.split('T').collect();
    if parts.len() != 2 {
        return String::new();
    }

    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    let time_parts: Vec<u32> = parts[1].split(':').filter_map(|s| s.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return String::new();
    }

    // Rough calculation (not accounting for timezones)
    let days_since_epoch = (date_parts[0] - 1970) * 365 + (date_parts[1] - 1) * 30 + date_parts[2];
    let secs = (days_since_epoch as u64) * 86400 + (time_parts[0] as u64) * 3600 + (time_parts[1] as u64) * 60;

    let msg_time = UNIX_EPOCH + Duration::from_secs(secs);
    let now = SystemTime::now();

    match now.duration_since(msg_time) {
        Ok(duration) => {
            let secs = duration.as_secs();
            if secs < 60 {
                "now".to_string()
            } else if secs < 3600 {
                format!("{}m", secs / 60)
            } else if secs < 86400 {
                format!("{}h", secs / 3600)
            } else {
                format!("{}d", secs / 86400)
            }
        }
        Err(_) => String::new(),
    }
}
