//! Instagram-themed loading spinner animations
//!
//! Provides animated spinners that match Instagram's brand colors,
//! with animated trailing dots for a polished loading experience.

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::colors::instagram;

/// An Instagram-themed spinner with animated dots
///
/// The spinner displays a braille animation in Instagram purple,
/// with the message dots animating: "Loading." → "Loading.." → "Loading..."
pub struct Spinner {
    progress: ProgressBar,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Finish and clear the spinner from the terminal
    pub fn finish_and_clear(mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.progress.finish_and_clear();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Create an Instagram-themed spinner with animated dots
///
/// The spinner uses Instagram's purple color and animates both
/// the spinner character and the trailing dots in the message.
///
/// # Example
/// ```
/// let spinner = create_spinner("Fetching inbox");
/// let result = client.get_inbox().await;
/// spinner.finish_and_clear();
/// ```
///
/// Note: Pass the message WITHOUT trailing dots - they will be animated automatically.
pub fn create_spinner(message: &str) -> Spinner {
    let spinner = ProgressBar::new_spinner();

    // Use Instagram purple for the spinner
    let (r, g, b) = instagram::PURPLE;

    // Smooth braille animation with Instagram purple color
    let style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template(&format!(
            "\x1b[38;2;{};{};{}m{{spinner}}\x1b[0m {{msg}}",
            r, g, b
        ))
        .expect("Invalid spinner template");

    spinner.set_style(style);

    // Remove trailing dots from message if present (we'll animate them)
    let base_message = message.trim_end_matches('.');

    // Set initial message
    spinner.set_message(format!("{}.", base_message));

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let spinner_clone = spinner.clone();
    let base_msg = base_message.to_string();

    // Spawn a thread to animate both spinner and dots
    let handle = thread::spawn(move || {
        let dots = [".", "..", "..."];
        let mut dot_index = 0;
        let mut tick_count = 0;

        while running_clone.load(Ordering::SeqCst) {
            // Tick the spinner (fast)
            spinner_clone.tick();

            // Update dots every 3 ticks (slower animation for dots)
            tick_count += 1;
            if tick_count >= 3 {
                tick_count = 0;
                dot_index = (dot_index + 1) % dots.len();
                spinner_clone.set_message(format!("{}{}", base_msg, dots[dot_index]));
            }

            thread::sleep(Duration::from_millis(80));
        }
    });

    Spinner {
        progress: spinner,
        running,
        handle: Some(handle),
    }
}
