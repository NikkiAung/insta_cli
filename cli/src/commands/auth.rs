//! Authentication commands with interactive prompts

use anyhow::Result;
use dialoguer::{Input, Password};

use crate::client::ApiClient;
use crate::colors::Theme;
use crate::spinner::create_spinner;

/// Interactive login with encrypted password
pub async fn login_interactive(client: &ApiClient) -> Result<()> {
    println!("{}", Theme::header("Instagram Login"));
    println!("{}", Theme::separator(40));
    println!(
        "{}",
        Theme::muted("Your password will be encrypted before transmission.")
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
    let spinner = create_spinner("Authenticating...");

    // Attempt login with encrypted password
    let result = client.login(&username, &password).await;
    spinner.finish_and_clear();

    match result {
        Ok(response) => {
            if response.success {
                println!("{} {}", Theme::check(), Theme::success("Login successful!"));
                if let Some(user) = response.user {
                    println!(
                        "  {} {} ({})",
                        Theme::muted("Logged in as:"),
                        Theme::username(&user.username),
                        user.full_name.unwrap_or_default()
                    );
                }
            } else {
                println!(
                    "{} {}",
                    Theme::cross(),
                    Theme::error(&response.message.unwrap_or("Login failed".to_string()))
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

/// Login with provided credentials (non-interactive)
pub async fn login_with_credentials(
    client: &ApiClient,
    username: &str,
    password: &str,
) -> Result<()> {
    let spinner = create_spinner("Authenticating...");

    let result = client.login(username, password).await;
    spinner.finish_and_clear();

    match result {
        Ok(response) => {
            if response.success {
                println!("{} {}", Theme::check(), Theme::success("Login successful!"));
                if let Some(user) = response.user {
                    println!(
                        "  {} {} ({})",
                        Theme::muted("Logged in as:"),
                        Theme::username(&user.username),
                        user.full_name.unwrap_or_default()
                    );
                }
            } else {
                println!(
                    "{} {}",
                    Theme::cross(),
                    Theme::error(&response.message.unwrap_or("Login failed".to_string()))
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

/// Logout from Instagram
pub async fn logout(client: &ApiClient) -> Result<()> {
    let spinner = create_spinner("Logging out...");

    client.logout().await?;
    spinner.finish_and_clear();

    println!("{} {}", Theme::check(), Theme::success("Logged out successfully"));
    Ok(())
}

/// Check authentication status
pub async fn status(client: &ApiClient) -> Result<()> {
    let spinner = create_spinner("Checking status...");

    let result = client.health().await;
    spinner.finish_and_clear();

    match result {
        Ok(health) => {
            println!("{}", Theme::header("Server Status"));
            println!("{}", Theme::separator(40));
            println!(
                "  {} {}",
                Theme::muted("Server:"),
                Theme::success(&health.status)
            );
            if health.authenticated {
                println!(
                    "  {} {} ({})",
                    Theme::muted("Status:"),
                    Theme::success("Authenticated"),
                    Theme::username(&health.username.unwrap_or_default())
                );
            } else {
                println!(
                    "  {} {}",
                    Theme::muted("Status:"),
                    Theme::warning("Not authenticated")
                );
            }
            Ok(())
        }
        Err(e) => {
            println!(
                "{} {} {}",
                Theme::cross(),
                Theme::error("Cannot connect to server:"),
                e
            );
            Err(e)
        }
    }
}

/// Show current logged-in user info
pub async fn show_me(client: &ApiClient) -> Result<()> {
    let spinner = create_spinner("Loading profile...");

    let result = client.health().await;
    spinner.finish_and_clear();

    match result {
        Ok(health) => {
            if health.authenticated {
                println!();
                println!("{}", Theme::header("Current User"));
                println!("{}", Theme::separator(40));
                println!(
                    "  {} {}",
                    Theme::muted("Username:"),
                    Theme::username(&format!("@{}", health.username.unwrap_or_default()))
                );
                println!();
            } else {
                println!(
                    "{} {}",
                    Theme::warn_icon(),
                    Theme::warning("Not logged in. Use 'ig login' first.")
                );
            }
            Ok(())
        }
        Err(e) => {
            println!(
                "{} {} {}",
                Theme::cross(),
                Theme::error("Cannot connect to server:"),
                e
            );
            Err(e)
        }
    }
}

/// Search for a user by username
pub async fn search_user(client: &ApiClient, query: &str) -> Result<()> {
    // Remove @ prefix if present
    let username = query.trim_start_matches('@');

    let spinner = create_spinner(&format!("Searching for @{}...", username));

    let result = client.search_user(username).await;
    spinner.finish_and_clear();

    match result {
        Ok(response) => {
            if let Some(user) = response.user {
                println!();
                println!("{}", Theme::header("User Found"));
                println!("{}", Theme::separator(40));
                println!(
                    "  {} {}",
                    Theme::muted("Username:"),
                    Theme::username(&format!("@{}", user.username))
                );
                if let Some(name) = user.full_name {
                    if !name.is_empty() {
                        println!("  {} {}", Theme::muted("Name:"), name);
                    }
                }
                if let Some(verified) = user.is_verified {
                    if verified {
                        println!("  {} {}", Theme::muted("Verified:"), Theme::blue("✓"));
                    }
                }
                if let Some(private) = user.is_private {
                    println!(
                        "  {} {}",
                        Theme::muted("Account:"),
                        if private { Theme::warning("Private") } else { Theme::success("Public") }
                    );
                }
                if let Some(followers) = user.follower_count {
                    println!("  {} {}", Theme::muted("Followers:"), Theme::accent(&format_count(followers)));
                }
                if let Some(following) = user.following_count {
                    println!("  {} {}", Theme::muted("Following:"), Theme::accent(&format_count(following)));
                }
                println!();
                println!(
                    "{}",
                    Theme::muted(&format!("Send message: ig send {} -m \"Hello!\"", user.username))
                );
            } else {
                println!(
                    "{} {}",
                    Theme::warn_icon(),
                    Theme::warning(&format!("User @{} not found", username))
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

/// Format large numbers (1000 -> 1K, 1000000 -> 1M)
fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
