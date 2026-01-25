//! Inbox and thread commands

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};

use crate::client::ApiClient;
use crate::colors::{Theme, instagram};
use crate::models::Thread;
use crate::commands::chat_with_user;
use crate::spinner::create_spinner;

/// Display inbox (list of conversations)
pub async fn show_inbox(client: &ApiClient, limit: u32, unread_only: bool) -> Result<()> {
    let spinner = create_spinner("Fetching inbox...");

    let response = client.get_inbox(limit).await;
    spinner.finish_and_clear();

    let response = response?;

    if !response.success {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&response.error.unwrap_or("Failed to fetch inbox".to_string()))
        );
        return Ok(());
    }

    let threads = response.threads.unwrap_or_default();

    // Filter to unread only if flag is set
    let threads: Vec<_> = if unread_only {
        threads.into_iter().filter(|t| t.has_unread.unwrap_or(false)).collect()
    } else {
        threads
    };

    if threads.is_empty() {
        if unread_only {
            println!("{}", Theme::muted("No unread conversations."));
        } else {
            println!("{}", Theme::muted("No conversations found."));
        }
        return Ok(());
    }

    println!();
    if unread_only {
        println!("{} {}", Theme::header("Inbox"), Theme::blue("(unread)"));
    } else {
        println!("{}", Theme::header("Inbox"));
    }
    println!("{}", Theme::separator(60));

    for (i, thread) in threads.iter().enumerate() {
        print_thread_summary(i + 1, thread);
    }

    println!("{}", Theme::separator(60));
    println!(
        "{}",
        Theme::muted(&format!("Showing {} conversations", threads.len()))
    );

    Ok(())
}

/// Watch mode - auto-refresh inbox every N seconds
pub async fn show_inbox_watch(client: &ApiClient, limit: u32, unread_only: bool, interval: u64) -> Result<()> {
    // Enable raw mode for keyboard detection
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Hide cursor
    execute!(stdout, cursor::Hide)?;

    loop {
        // Clear screen
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // Fetch inbox
        let response = client.get_inbox(limit).await;

        match response {
            Ok(response) => {
                if !response.success {
                    writeln!(
                        stdout,
                        "\r{} {}",
                        Theme::cross(),
                        Theme::error(&response.error.unwrap_or("Failed to fetch inbox".to_string()))
                    )?;
                } else {
                    let threads = response.threads.unwrap_or_default();

                    // Filter to unread only if flag is set
                    let threads: Vec<_> = if unread_only {
                        threads.into_iter().filter(|t| t.has_unread.unwrap_or(false)).collect()
                    } else {
                        threads
                    };

                    // Header
                    writeln!(stdout, "\r")?;
                    if unread_only {
                        writeln!(stdout, "\r{} {}", Theme::header("Inbox"), Theme::blue("(unread)"))?;
                    } else {
                        writeln!(stdout, "\r{}", Theme::header("Inbox"))?;
                    }
                    writeln!(stdout, "\r{}", Theme::separator(60))?;

                    if threads.is_empty() {
                        if unread_only {
                            writeln!(stdout, "\r{}", Theme::muted("No unread conversations."))?;
                        } else {
                            writeln!(stdout, "\r{}", Theme::muted("No conversations found."))?;
                        }
                    } else {
                        for (i, thread) in threads.iter().enumerate() {
                            print_thread_summary_watch(&mut stdout, i + 1, thread)?;
                        }
                    }

                    writeln!(stdout, "\r{}", Theme::separator(60))?;
                    writeln!(
                        stdout,
                        "\r{} {} {}",
                        Theme::muted(&format!("Showing {} conversations", threads.len())),
                        Theme::muted("•"),
                        Theme::muted(&format!("Refreshing every {}s", interval))
                    )?;
                }
            }
            Err(e) => {
                writeln!(stdout, "\r{} {}", Theme::cross(), Theme::error(&format!("{}", e)))?;
            }
        }

        writeln!(stdout, "\r")?;
        writeln!(stdout, "\r{}", Theme::muted("Press 'q' to quit"))?;
        stdout.flush()?;

        // Wait for interval, but check for 'q' key every 100ms
        let check_interval = Duration::from_millis(100);
        let total_checks = (interval * 1000) / 100;

        for _ in 0..total_checks {
            if event::poll(check_interval)? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc) {
                            // Restore terminal
                            execute!(stdout, cursor::Show)?;
                            terminal::disable_raw_mode()?;
                            println!("\r");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

/// Print thread summary for watch mode (with \r for raw mode)
fn print_thread_summary_watch(stdout: &mut io::Stdout, index: usize, thread: &Thread) -> Result<()> {
    let username = thread.users.first().map(|u| u.username.as_str()).unwrap_or("unknown");

    let title = thread
        .thread_title
        .clone()
        .unwrap_or_else(|| username.to_string());

    let last_msg = thread
        .last_message_text
        .as_ref()
        .map(|s| {
            if s.len() > 40 {
                format!("{}...", &s[..37])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "[no messages]".to_string());

    let time_ago = thread
        .last_message_timestamp
        .as_ref()
        .map(|t| format_time_ago_colored(t))
        .unwrap_or_else(|| Theme::timestamp("").to_string());

    let unread_indicator = if thread.has_unread.unwrap_or(false) {
        format!("{} ", Theme::unread_dot())
    } else {
        "  ".to_string()
    };

    writeln!(
        stdout,
        "\r{}{:>2}. {} {} {}",
        unread_indicator,
        index,
        title,
        Theme::muted(&format!("@{}", username)),
        time_ago
    )?;
    writeln!(stdout, "\r     └ {}", Theme::muted(&last_msg))?;

    Ok(())
}

/// Display a specific thread with messages
pub async fn show_thread(client: &ApiClient, thread_id: &str, limit: u32) -> Result<()> {
    let spinner = create_spinner("Fetching messages...");

    let response = client.get_thread(thread_id, limit).await;
    spinner.finish_and_clear();

    let response = response?;

    if !response.success {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&response.error.unwrap_or("Failed to fetch thread".to_string()))
        );
        return Ok(());
    }

    let thread = match response.thread {
        Some(t) => t,
        None => {
            println!("{}", Theme::muted("Thread not found."));
            return Ok(());
        }
    };

    println!();
    let participants: Vec<&str> = thread.users.iter().map(|u| u.username.as_str()).collect();
    println!(
        "{} {}",
        Theme::header("Conversation with:"),
        Theme::username(&participants.join(", "))
    );
    println!("{}", Theme::separator(60));

    let messages = thread.messages.unwrap_or_default();

    if messages.is_empty() {
        println!("{}", Theme::muted("No messages in this thread."));
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
            "{} {}",
            Theme::pink(sender),
            Theme::timestamp(&time)
        );
        println!("  {}", text);
        println!();
    }

    println!("{}", Theme::separator(60));
    println!(
        "{}",
        Theme::muted(&format!("Thread ID: {}", thread_id))
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
    let unread = if thread.has_unread.unwrap_or(false) {
        format!("{}", Theme::unread_dot())
    } else {
        " ".to_string()
    };

    // Time (colored based on recency)
    let time = thread
        .last_message_timestamp
        .as_ref()
        .map(|t| format_time_ago_colored(t))
        .unwrap_or_default();

    // Show: "1. Display Name (@username) 13d"
    println!(
        "{:>3}. {} {} {} {}",
        Theme::muted(&index.to_string()),
        Theme::orange(&title),
        Theme::username(&format!("@{}", username)),
        time,  // Already colored
        unread
    );
    println!("     {} {}", Theme::muted("└"), preview);
}

/// Format ISO timestamp to relative time (plain string)
fn format_time_ago(timestamp: &str) -> String {
    let (text, _) = parse_time_ago(timestamp);
    text
}

/// Format ISO timestamp to colored relative time
fn format_time_ago_colored(timestamp: &str) -> String {
    let (text, age_type) = parse_time_ago(timestamp);

    match age_type {
        TimeAge::Now => format!("{}", text.truecolor(46, 204, 113)), // Green
        TimeAge::Minutes => {
            let (r, g, b) = instagram::BLUE;
            format!("{}", text.truecolor(r, g, b))
        }
        TimeAge::Hours => {
            let (r, g, b) = instagram::ORANGE;
            format!("{}", text.truecolor(r, g, b))
        }
        TimeAge::Days => {
            let (r, g, b) = instagram::LIGHT_GRAY;
            format!("{}", text.truecolor(r, g, b))
        }
        TimeAge::Unknown => text,
    }
}

/// Time age categories for coloring
enum TimeAge {
    Now,
    Minutes,
    Hours,
    Days,
    Unknown,
}

/// Parse ISO timestamp to relative time with age category
fn parse_time_ago(timestamp: &str) -> (String, TimeAge) {
    use chrono::{Local, NaiveDateTime, TimeZone};

    // Parse "2026-01-24T16:07:11" format (ISO 8601 without timezone)
    let naive = match NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => dt,
        Err(_) => return (String::new(), TimeAge::Unknown),
    };

    // Treat the timestamp as local time
    let msg_time = match Local.from_local_datetime(&naive).single() {
        Some(dt) => dt,
        None => return (String::new(), TimeAge::Unknown),
    };

    let now = Local::now();
    let duration = now.signed_duration_since(msg_time);
    let secs = duration.num_seconds();

    if secs < 0 {
        // Future timestamp (shouldn't happen, but handle gracefully)
        return ("now".to_string(), TimeAge::Now);
    }

    let secs = secs as u64;
    if secs < 60 {
        ("now".to_string(), TimeAge::Now)
    } else if secs < 3600 {
        (format!("{}m", secs / 60), TimeAge::Minutes)
    } else if secs < 86400 {
        (format!("{}h", secs / 3600), TimeAge::Hours)
    } else {
        (format!("{}d", secs / 86400), TimeAge::Days)
    }
}

/// Open chat by inbox number (1, 2, 3...)
pub async fn open_by_number(client: &ApiClient, number: usize) -> Result<()> {
    if number == 0 {
        println!("{} {}", Theme::cross(), Theme::error("Number must be 1 or greater"));
        return Ok(());
    }

    let spinner = create_spinner("Fetching inbox...");

    let response = client.get_inbox(number as u32).await;
    spinner.finish_and_clear();

    let response = response?;

    if !response.success {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&response.error.unwrap_or("Failed to fetch inbox".to_string()))
        );
        return Ok(());
    }

    let threads = response.threads.unwrap_or_default();

    if number > threads.len() {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&format!("No conversation at position {}. You have {} conversations.", number, threads.len()))
        );
        return Ok(());
    }

    // Get the thread at position (1-indexed)
    let thread = &threads[number - 1];
    let username = thread.users.first().map(|u| u.username.as_str()).unwrap_or("unknown");

    // Extract all usernames for tab completion
    let usernames: Vec<String> = threads
        .iter()
        .flat_map(|t| t.users.iter().map(|u| u.username.clone()))
        .collect();

    // Start chat with this user
    chat_with_user(client, username, usernames).await
}

/// Show thread by ID or @username
pub async fn show_thread_or_user(client: &ApiClient, target: &str, limit: u32) -> Result<()> {
    // Check if target starts with @ (username)
    if target.starts_with('@') {
        let username = &target[1..]; // Remove @ prefix
        show_thread_by_username(client, username, limit).await
    } else {
        // Assume it's a thread ID
        show_thread(client, target, limit).await
    }
}

/// Show thread by username (finds the thread first)
async fn show_thread_by_username(client: &ApiClient, username: &str, limit: u32) -> Result<()> {
    let spinner = create_spinner(&format!("Finding conversation with @{}...", username));

    // Fetch inbox to find the thread
    let response = client.get_inbox(100).await;
    spinner.finish_and_clear();

    let response = response?;

    if !response.success {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&response.error.unwrap_or("Failed to fetch inbox".to_string()))
        );
        return Ok(());
    }

    let threads = response.threads.unwrap_or_default();

    // Find thread with this username
    let thread = threads.iter().find(|t| {
        t.users.iter().any(|u| u.username.eq_ignore_ascii_case(username))
    });

    match thread {
        Some(t) => {
            show_thread(client, &t.id, limit).await
        }
        None => {
            println!(
                "{} {}",
                Theme::warn_icon(),
                Theme::warning(&format!("No conversation found with @{}", username))
            );
            Ok(())
        }
    }
}

/// Interactive inbox with arrow key navigation
pub async fn show_inbox_interactive(client: &ApiClient, limit: u32) -> Result<()> {
    let spinner = create_spinner("Fetching inbox");

    let response = client.get_inbox(limit).await;
    spinner.finish_and_clear();

    let response = response?;

    if !response.success {
        println!(
            "{} {}",
            Theme::cross(),
            Theme::error(&response.error.unwrap_or("Failed to fetch inbox".to_string()))
        );
        return Ok(());
    }

    let threads = response.threads.unwrap_or_default();

    if threads.is_empty() {
        println!("{}", Theme::muted("No conversations found."));
        return Ok(());
    }

    // Enter raw mode for keyboard input
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Hide cursor
    execute!(stdout, cursor::Hide)?;

    let mut selected: usize = 0;
    let mut should_open: Option<usize> = None;

    loop {
        // Clear screen and draw
        execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

        // Header
        let header = format!("{}", Theme::header("Inbox"));
        writeln!(stdout, "\r\n{}", header)?;
        writeln!(stdout, "\r{}", Theme::separator(60))?;

        // Draw threads
        for (i, thread) in threads.iter().enumerate() {
            let is_selected = i == selected;
            print_thread_interactive(&mut stdout, i + 1, thread, is_selected)?;
        }

        // Footer
        writeln!(stdout, "\r{}", Theme::separator(60))?;
        writeln!(
            stdout,
            "\r{}",
            Theme::muted("↑/↓: Navigate  Enter: Open chat  q: Quit")
        )?;

        stdout.flush()?;

        // Handle input
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < threads.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        should_open = Some(selected);
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;

    // Open selected chat if user pressed Enter
    if let Some(idx) = should_open {
        let thread = &threads[idx];
        let username = thread.users.first().map(|u| u.username.as_str()).unwrap_or("unknown");

        // Extract all usernames for tab completion
        let usernames: Vec<String> = threads
            .iter()
            .flat_map(|t| t.users.iter().map(|u| u.username.clone()))
            .collect();

        chat_with_user(client, username, usernames).await?;
    }

    Ok(())
}

/// Print a thread summary for interactive view
fn print_thread_interactive(
    stdout: &mut io::Stdout,
    index: usize,
    thread: &Thread,
    is_selected: bool,
) -> Result<()> {
    let username = thread.users.first().map(|u| u.username.as_str()).unwrap_or("unknown");

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
    let unread = if thread.has_unread.unwrap_or(false) {
        format!("{}", Theme::unread_dot())
    } else {
        " ".to_string()
    };

    // Time (colored based on recency)
    let time = thread
        .last_message_timestamp
        .as_ref()
        .map(|t| format_time_ago_colored(t))
        .unwrap_or_default();

    // Selection indicator and highlighting
    let (indicator, highlight_start, highlight_end) = if is_selected {
        let (r, g, b) = instagram::PINK;
        (
            format!("\x1b[38;2;{};{};{}m►\x1b[0m", r, g, b),
            format!("\x1b[48;2;60;60;60m"), // Dark background for highlight
            "\x1b[0m".to_string(),
        )
    } else {
        (" ".to_string(), String::new(), String::new())
    };

    writeln!(
        stdout,
        "\r{} {}{:>2}. {} {} {} {}{}",
        indicator,
        highlight_start,
        index,
        Theme::orange(&title),
        Theme::username(&format!("@{}", username)),
        time,  // Already colored
        unread,
        highlight_end
    )?;
    writeln!(
        stdout,
        "\r       {}{} {}{}",
        highlight_start,
        Theme::muted("└"),
        preview,
        highlight_end
    )?;

    Ok(())
}
