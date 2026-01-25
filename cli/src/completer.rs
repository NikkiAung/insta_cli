//! Tab completion for usernames from recent conversations

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;

use crate::colors::instagram;

/// Username completer that suggests usernames from recent conversations
#[derive(Clone)]
pub struct UsernameCompleter {
    usernames: Vec<String>,
}

impl UsernameCompleter {
    pub fn new(usernames: Vec<String>) -> Self {
        Self { usernames }
    }
}

impl Completer for UsernameCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let input = &line[..pos];

        // Find the start of the current word (after @ or space)
        let word_start = input
            .rfind(|c: char| c == '@' || c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &input[word_start..];

        if prefix.is_empty() {
            return Ok((pos, vec![]));
        }

        let matches: Vec<Pair> = self
            .usernames
            .iter()
            .filter(|name| name.to_lowercase().starts_with(&prefix.to_lowercase()))
            .map(|name| Pair {
                display: format!("@{}", name),
                replacement: name.clone(),
            })
            .collect();

        Ok((word_start, matches))
    }
}

impl Hinter for UsernameCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        // Find the start of the current word
        let word_start = line
            .rfind(|c: char| c == '@' || c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &line[word_start..];

        if prefix.is_empty() {
            return None;
        }

        // Find first matching username
        self.usernames
            .iter()
            .find(|name| name.to_lowercase().starts_with(&prefix.to_lowercase()))
            .map(|name| name[prefix.len()..].to_string())
    }
}

impl Highlighter for UsernameCompleter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Gray color for hints
        Cow::Owned(format!("\x1b[38;2;142;142;142m{}\x1b[0m", hint))
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Highlight @usernames in pink
        let (r, g, b) = instagram::PINK;
        let mut result = String::new();
        let mut in_username = false;
        let mut current_word = String::new();

        for c in line.chars() {
            if c == '@' {
                in_username = true;
                current_word.push(c);
            } else if in_username && (c.is_alphanumeric() || c == '_') {
                current_word.push(c);
            } else {
                if in_username && !current_word.is_empty() {
                    result.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, current_word));
                    current_word.clear();
                }
                in_username = false;
                result.push(c);
            }
        }

        // Handle trailing username
        if in_username && !current_word.is_empty() {
            result.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, current_word));
        }

        if result.is_empty() {
            Cow::Borrowed(line)
        } else {
            Cow::Owned(result)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl Validator for UsernameCompleter {}

impl Helper for UsernameCompleter {}

/// Create a readline editor with username completion
pub fn create_editor(usernames: Vec<String>) -> Editor<UsernameCompleter, DefaultHistory> {
    let completer = UsernameCompleter::new(usernames);
    let mut editor = Editor::new().expect("Failed to create editor");
    editor.set_helper(Some(completer));
    editor
}
