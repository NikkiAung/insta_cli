//! Instagram-themed loading spinner animations
//!
//! Provides animated spinners that match Instagram's brand colors,
//! cycling through the iconic gradient: Purple → Pink → Orange → Yellow

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::colors::instagram;

/// Instagram gradient colors for the spinner
const GRADIENT: [(u8, u8, u8); 4] = [
    instagram::PURPLE,  // #833AB4
    instagram::PINK,    // #E1306C
    instagram::ORANGE,  // #F77737
    instagram::YELLOW,  // #FCAF45
];

/// Spinner characters for smooth animation
const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// An Instagram-themed spinner with gradient colors and animated dots
///
/// The spinner cycles through Instagram's signature gradient colors
/// (purple → pink → orange → yellow) while also animating the trailing dots.
pub struct Spinner {
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
        // Clear the line and show cursor
        print!("\r\x1b[K\x1b[?25h");
        let _ = io::stdout().flush();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // Ensure cursor is shown if dropped unexpectedly
        print!("\x1b[?25h");
        let _ = io::stdout().flush();
    }
}

/// Create an Instagram gradient spinner with animated dots
///
/// The spinner cycles through Instagram's brand colors while animating:
/// - Spinner character: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
/// - Gradient colors: Purple → Pink → Orange → Yellow
/// - Trailing dots: . → .. → ...
///
/// # Example
/// ```
/// let spinner = create_spinner("Fetching inbox");
/// let result = client.get_inbox().await;
/// spinner.finish_and_clear();
/// ```
pub fn create_spinner(message: &str) -> Spinner {
    // Remove trailing dots from message (we'll animate them)
    let base_message = message.trim_end_matches('.').to_string();

    // Hide cursor
    print!("\x1b[?25l");
    let _ = io::stdout().flush();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = thread::spawn(move || {
        // Pad dots to same length to avoid leftover characters
        let dots = [".  ", ".. ", "..."];
        let mut spinner_index = 0;
        let mut color_index = 0;
        let mut dot_index = 0;
        let mut tick_count = 0;

        while running_clone.load(Ordering::SeqCst) {
            // Get current color from gradient
            let (r, g, b) = GRADIENT[color_index];

            // Get current spinner character
            let spinner_char = SPINNER_CHARS[spinner_index];

            // Build the line with colored spinner
            let line = format!(
                "\r\x1b[38;2;{};{};{}m{}\x1b[0m {}{}",
                r, g, b,
                spinner_char,
                base_message,
                dots[dot_index]
            );

            print!("{}", line);
            let _ = io::stdout().flush();

            // Advance spinner (every tick)
            spinner_index = (spinner_index + 1) % SPINNER_CHARS.len();

            // Advance color (every 2 spinner frames for smooth gradient)
            tick_count += 1;
            if tick_count % 2 == 0 {
                color_index = (color_index + 1) % GRADIENT.len();
            }

            // Advance dots every ~320ms (4 ticks * 80ms)
            if tick_count % 4 == 0 {
                dot_index = (dot_index + 1) % dots.len();
            }

            thread::sleep(Duration::from_millis(80));
        }
    });

    Spinner {
        running,
        handle: Some(handle),
    }
}
