use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Instant;

/// How much autonomy the agent has
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AutonomyLevel {
    /// Read-only: can observe but not act
    ReadOnly,
    /// Supervised: acts but requires approval for risky operations
    #[default]
    Supervised,
    /// Full: autonomous execution within policy bounds
    Full,
}

/// Risk score for shell command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRiskLevel {
    Low,
    Medium,
    High,
}

/// Classifies whether a tool operation is read-only or side-effecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOperation {
    Read,
    Act,
}

/// Sliding-window action tracker for rate limiting.
#[derive(Debug)]
pub struct ActionTracker {
    /// Timestamps of recent actions (kept within the last hour).
    actions: Mutex<Vec<Instant>>,
}

impl ActionTracker {
    pub fn new() -> Self {
        Self {
            actions: Mutex::new(Vec::new()),
        }
    }

    /// Record an action and return the current count within the window.
    pub fn record(&self) -> usize {
        let mut actions = self.actions.lock();
        let cutoff = Instant::now()
            .checked_sub(std::time::Duration::from_secs(3600))
            .unwrap_or_else(Instant::now);
        actions.retain(|t| *t > cutoff);
        actions.push(Instant::now());
        actions.len()
    }

    /// Count of actions in the current window without recording.
    pub fn count(&self) -> usize {
        let mut actions = self.actions.lock();
        let cutoff = Instant::now()
            .checked_sub(std::time::Duration::from_secs(3600))
            .unwrap_or_else(Instant::now);
        actions.retain(|t| *t > cutoff);
        actions.len()
    }
}

impl Clone for ActionTracker {
    fn clone(&self) -> Self {
        let actions = self.actions.lock();
        Self {
            actions: Mutex::new(actions.clone()),
        }
    }
}

/// Security policy enforced on all tool executions
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub autonomy: AutonomyLevel,
    pub workspace_dir: PathBuf,
    pub workspace_only: bool,
    pub allowed_commands: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub allowed_roots: Vec<PathBuf>,
    pub max_actions_per_hour: u32,
    pub max_cost_per_day_cents: u32,
    pub require_approval_for_medium_risk: bool,
    pub block_high_risk_commands: bool,
    pub(crate) live_allowed_commands: Arc<Mutex<Vec<String>>>,
    pub(crate) live_allowed_commands_overridden: Arc<AtomicBool>,
    pub(crate) live_require_approval_for_medium_risk: Arc<AtomicBool>,
    pub(crate) live_block_high_risk_commands: Arc<AtomicBool>,
    pub shell_env_passthrough: Vec<String>,
    pub tracker: ActionTracker,
    /// Runtime-updatable limit. `u32::MAX` means "use the static `max_actions_per_hour`".
    pub(crate) live_limit: Arc<AtomicU32>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            autonomy: AutonomyLevel::Supervised,
            workspace_dir: PathBuf::from("."),
            workspace_only: true,
            allowed_commands: vec![
                "git".into(),
                "npm".into(),
                "cargo".into(),
                "ls".into(),
                "cat".into(),
                "grep".into(),
                "find".into(),
                "echo".into(),
                "pwd".into(),
                "wc".into(),
                "head".into(),
                "tail".into(),
                "date".into(),
                "osascript".into(),
                "sleep".into(),
            ],
            forbidden_paths: vec![
                // System directories (blocked even when workspace_only=false)
                "/etc".into(),
                "/root".into(),
                "/home".into(),
                "/usr".into(),
                "/bin".into(),
                "/sbin".into(),
                "/lib".into(),
                "/opt".into(),
                "/boot".into(),
                "/dev".into(),
                "/proc".into(),
                "/sys".into(),
                "/var".into(),
                // Sensitive dotfiles
                "~/.ssh".into(),
                "~/.gnupg".into(),
                "~/.aws".into(),
                "~/.config".into(),
            ],
            allowed_roots: Vec::new(),
            max_actions_per_hour: 20,
            max_cost_per_day_cents: 500,
            require_approval_for_medium_risk: true,
            block_high_risk_commands: true,
            live_allowed_commands: Arc::new(Mutex::new(vec![
                "git".into(),
                "npm".into(),
                "cargo".into(),
                "ls".into(),
                "cat".into(),
                "grep".into(),
                "find".into(),
                "echo".into(),
                "pwd".into(),
                "wc".into(),
                "head".into(),
                "tail".into(),
                "date".into(),
                "osascript".into(),
                "sleep".into(),
            ])),
            live_allowed_commands_overridden: Arc::new(AtomicBool::new(false)),
            live_require_approval_for_medium_risk: Arc::new(AtomicBool::new(true)),
            live_block_high_risk_commands: Arc::new(AtomicBool::new(true)),
            shell_env_passthrough: vec![],
            tracker: ActionTracker::new(),
            live_limit: Arc::new(AtomicU32::new(u32::MAX)),
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Env vars permitted in `$VAR` / `"$VAR/..."` patterns for shell policy checks.
/// Unknown `$FOO` remains blocked so paths cannot hide behind indirection (e.g.
/// `cat $SECRET_FILE`). Expansions here must match [`expand_user_path`].
const SAFE_SHELL_PATH_VARS: &[&str] = &["HOME", "TMPDIR", "PWD"];

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(stripped);
        }
    }

    if let Some(rest) = path.strip_prefix("$HOME/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    if path == "$HOME" {
        return home_dir().unwrap_or_else(|| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("$TMPDIR/") {
        if let Ok(tmp) = std::env::var("TMPDIR") {
            return PathBuf::from(tmp).join(rest);
        }
    }
    if path == "$TMPDIR" {
        return std::env::var("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("$PWD/") {
        if let Ok(pwd) = std::env::var("PWD") {
            return PathBuf::from(pwd).join(rest);
        }
    }
    if path == "$PWD" {
        return std::env::var("PWD")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(path));
    }

    PathBuf::from(path)
}

// ── Shell Command Parsing Utilities ───────────────────────────────────────
// These helpers implement a minimal quote-aware shell lexer. They exist
// because security validation must reason about the *structure* of a
// command (separators, operators, quoting) rather than treating it as a
// flat string — otherwise an attacker could hide dangerous sub-commands
// inside quoted arguments or chained operators.
/// Skip leading environment variable assignments (e.g. `FOO=bar cmd args`).
/// Returns the remainder starting at the first non-assignment word.
fn skip_env_assignments(s: &str) -> &str {
    let mut rest = s;
    loop {
        let Some(word) = rest.split_whitespace().next() else {
            return rest;
        };
        // Environment assignment: contains '=' and starts with a letter or underscore
        if word.contains('=')
            && word
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            // Advance past this word
            rest = rest[word.len()..].trim_start();
        } else {
            return rest;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteState {
    None,
    Single,
    Double,
}

#[derive(Debug, Clone)]
struct QuotedHeredocState {
    delimiter: String,
    allow_tab_indentation: bool,
    waiting_for_body: bool,
}

/// Parse a quoted heredoc opener starting right after `<<`.
///
/// Supported forms:
/// - `<<'DELIM'`
/// - `<<"DELIM"`
/// - `<<-'DELIM'`
/// - `<<-"DELIM"`
fn parse_quoted_heredoc_opener(
    chars: &[char],
    start: usize,
) -> Option<(usize, String, bool)> {
    let mut i = start;
    let mut allow_tab_indentation = false;

    if chars.get(i) == Some(&'-') {
        allow_tab_indentation = true;
        i += 1;
    }

    while i < chars.len() && chars[i].is_ascii_whitespace() && chars[i] != '\n' {
        i += 1;
    }

    let quote = *chars.get(i)?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    i += 1;

    let mut delimiter = String::new();
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];

        if quote == '"' {
            if escaped {
                delimiter.push(ch);
                escaped = false;
                i += 1;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                i += 1;
                continue;
            }
        }

        if ch == quote {
            break;
        }

        delimiter.push(ch);
        i += 1;
    }

    if delimiter.is_empty() || i >= chars.len() || chars[i] != quote {
        return None;
    }

    i += 1;
    Some((i, delimiter, allow_tab_indentation))
}

/// Remove quoted heredoc bodies from a command string so character-level
/// checks (unquoted `>`, `<`, `` ` ``, `$(`, `&`) do not fire on script body
/// content. Only bodies opened with a quoted delimiter (`<<'D'` / `<<"D"`) are
/// stripped; unquoted openers (`<<EOF`) are left in place and remain blocked.
fn strip_heredoc_bodies(command: &str) -> String {
    let chars: Vec<char> = command.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    let mut quote = QuoteState::None;
    let mut escaped = false;
    let mut heredoc: Option<QuotedHeredocState> = None;

    while i < chars.len() {
        if let Some(state) = heredoc.as_mut() {
            if state.waiting_for_body {
                let ch = chars[i];
                i += 1;
                if ch == '\n' {
                    state.waiting_for_body = false;
                }
                continue;
            }

            let line_start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            let line: String = chars[line_start..i].iter().collect();
            let normalized = line.trim_end_matches('\r');
            let is_terminator = if state.allow_tab_indentation {
                normalized.trim_start_matches('\t') == state.delimiter
            } else {
                normalized == state.delimiter
            };
            if is_terminator {
                heredoc = None;
            }
            if i < chars.len() && chars[i] == '\n' {
                i += 1;
            }
            continue;
        }

        let ch = chars[i];
        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
                out.push(ch);
                i += 1;
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                }
                out.push(ch);
                i += 1;
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '\'' {
                    quote = QuoteState::Single;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::Double;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '<' && chars.get(i + 1) == Some(&'<') {
                    if let Some((after_opener, delimiter, allow_tab_indentation)) =
                        parse_quoted_heredoc_opener(&chars, i + 2)
                    {
                        out.extend(chars[i..after_opener].iter().copied());
                        heredoc = Some(QuotedHeredocState {
                            delimiter,
                            allow_tab_indentation,
                            waiting_for_body: true,
                        });
                        i = after_opener;
                        continue;
                    }
                }
                out.push(ch);
                i += 1;
            }
        }
    }

    out
}

/// Split a shell command into sub-commands by unquoted separators.
///
/// Separators:
/// - `;` and newline
/// - `|`
/// - `&&`, `||`
///
/// Characters inside single or double quotes are treated as literals, so
/// `sqlite3 db "SELECT 1; SELECT 2;"` remains a single segment.
///
/// Quoted heredoc bodies are also treated as literal payload so embedded
/// newlines/operators do not get split into separate command segments.
fn split_unquoted_segments(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote = QuoteState::None;
    let mut escaped = false;
    let mut heredoc: Option<QuotedHeredocState> = None;
    let chars: Vec<char> = command.chars().collect();
    let mut i = 0usize;

    let push_segment = |segments: &mut Vec<String>, current: &mut String| {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            segments.push(trimmed.to_string());
        }
        current.clear();
    };

    while i < chars.len() {
        if let Some(state) = heredoc.as_mut() {
            if state.waiting_for_body {
                let ch = chars[i];
                current.push(ch);
                i += 1;
                if ch == '\n' {
                    state.waiting_for_body = false;
                }
                continue;
            }

            let line_start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }

            let line: String = chars[line_start..i].iter().collect();
            let normalized_line = line.trim_end_matches('\r');
            let is_terminator = if state.allow_tab_indentation {
                let without_tabs = normalized_line.trim_start_matches('\t');
                without_tabs == state.delimiter
            } else {
                normalized_line == state.delimiter
            };

            current.push_str(&line);
            if is_terminator {
                heredoc = None;
                continue;
            }

            if i < chars.len() && chars[i] == '\n' {
                current.push('\n');
                i += 1;
            }

            continue;
        }

        let ch = chars[i];
        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
                current.push(ch);
                i += 1;
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    current.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    current.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                }
                current.push(ch);
                i += 1;
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    current.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    current.push(ch);
                    i += 1;
                    continue;
                }

                match ch {
                    '\'' => {
                        quote = QuoteState::Single;
                        current.push(ch);
                        i += 1;
                    }
                    '"' => {
                        quote = QuoteState::Double;
                        current.push(ch);
                        i += 1;
                    }
                    '<' => {
                        if chars.get(i + 1) == Some(&'<') {
                            if let Some((after_opener, delimiter, allow_tab_indentation)) =
                                parse_quoted_heredoc_opener(&chars, i + 2)
                            {
                                current.extend(chars[i..after_opener].iter().copied());
                                heredoc = Some(QuotedHeredocState {
                                    delimiter,
                                    allow_tab_indentation,
                                    waiting_for_body: true,
                                });
                                i = after_opener;
                                continue;
                            }
                        }
                        current.push(ch);
                        i += 1;
                    }
                    ';' | '\n' => {
                        push_segment(&mut segments, &mut current);
                        i += 1;
                    }
                    '|' => {
                        if chars.get(i + 1) == Some(&'|') {
                            // Consume full `||`; both characters are separators.
                            i += 1;
                        }
                        push_segment(&mut segments, &mut current);
                        i += 1;
                    }
                    '&' => {
                        if chars.get(i + 1) == Some(&'&') {
                            // `&&` is a separator; single `&` is handled separately.
                            i += 1;
                            push_segment(&mut segments, &mut current);
                        } else {
                            current.push(ch);
                        }
                        i += 1;
                    }
                    _ => {
                        current.push(ch);
                        i += 1;
                    }
                }
            }
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }

    segments
}

/// Detect a single unquoted `&` operator (background/chain). `&&` is allowed.
///
/// We treat any standalone `&` as unsafe in policy validation because it can
/// chain hidden sub-commands and escape foreground timeout expectations.
fn contains_unquoted_single_ampersand(command: &str) -> bool {
    let mut quote = QuoteState::None;
    let mut escaped = false;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                }
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                match ch {
                    '\'' => quote = QuoteState::Single,
                    '"' => quote = QuoteState::Double,
                    '&' => {
                        if chars.next_if_eq(&'&').is_none() {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    false
}

/// Detect an unquoted character in a shell command.
fn contains_unquoted_char(command: &str, target: char) -> bool {
    let mut quote = QuoteState::None;
    let mut escaped = false;

    for ch in command.chars() {
        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                }
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                match ch {
                    '\'' => quote = QuoteState::Single,
                    '"' => quote = QuoteState::Double,
                    _ if ch == target => return true,
                    _ => {}
                }
            }
        }
    }

    false
}

/// True if the command uses `$` expansion that must stay blocked.
///
/// Allows only a small set of path-safe variables ([`SAFE_SHELL_PATH_VARS`]) so
/// commands like `mkdir -p "$HOME/Downloads/x"` work while `cat $SECRET_FILE`
/// and `$(...)` / `${...}` remain blocked. Escaped dollars (`\$`) and literals
/// inside single quotes are ignored.
fn contains_unquoted_shell_variable_expansion(command: &str) -> bool {
    let mut quote = QuoteState::None;
    let mut escaped = false;
    let chars: Vec<char> = command.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];

        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
                continue;
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                    continue;
                }
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '\'' {
                    quote = QuoteState::Single;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::Double;
                    continue;
                }
            }
        }

        if ch != '$' {
            continue;
        }

        let Some(next) = chars.get(i + 1).copied() else {
            continue;
        };

        match next {
            '{' | '(' => return true,
            c if c.is_ascii_digit() => return true,
            '*' | '@' | '#' | '!' | '$' | '?' | '-' => return true,
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut j = i + 2;
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let name: String = chars[i + 1..j].iter().collect();
                if SAFE_SHELL_PATH_VARS.contains(&name.as_str()) {
                    continue;
                }
                return true;
            }
            _ => return true,
        }
    }

    false
}

/// Unquoted `<` used for input redirection (`< file`) must stay blocked.
///
/// Allows `<<'DELIM'` / `<<"DELIM"` / `<<-'DELIM'` openers (quoted heredoc
/// delimiter only) so inline scripts like `python3 - <<'PY'` pass the gate.
/// Unquoted delimiters (`<<EOF`) remain blocked.
fn contains_unquoted_dangerous_lt(command: &str) -> bool {
    let chars: Vec<char> = command.chars().collect();
    let mut i = 0usize;
    let mut quote = QuoteState::None;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];
        match quote {
            QuoteState::Single => {
                if ch == '\'' {
                    quote = QuoteState::None;
                }
                i += 1;
            }
            QuoteState::Double => {
                if escaped {
                    escaped = false;
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::None;
                }
                i += 1;
            }
            QuoteState::None => {
                if escaped {
                    escaped = false;
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if ch == '\'' {
                    quote = QuoteState::Single;
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    quote = QuoteState::Double;
                    i += 1;
                    continue;
                }
                if ch == '<' {
                    if chars.get(i + 1) == Some(&'<') {
                        let mut j = i + 2;
                        if chars.get(j) == Some(&'-') {
                            j += 1;
                        }
                        while j < chars.len() && chars[j].is_ascii_whitespace() {
                            j += 1;
                        }
                        if let Some(&delim_ch) = chars.get(j) {
                            if delim_ch == '\'' {
                                j += 1;
                                while j < chars.len() && chars[j] != '\'' {
                                    j += 1;
                                }
                                if j < chars.len() {
                                    i = j + 1;
                                    continue;
                                }
                            } else if delim_ch == '"' {
                                j += 1;
                                let mut esc = false;
                                let mut closed = false;
                                while j < chars.len() {
                                    if esc {
                                        esc = false;
                                    } else if chars[j] == '\\' {
                                        esc = true;
                                    } else if chars[j] == '"' {
                                        j += 1;
                                        closed = true;
                                        break;
                                    }
                                    j += 1;
                                }
                                if closed {
                                    i = j;
                                    continue;
                                }
                            }
                        }
                    }
                    return true;
                }
                i += 1;
            }
        }
    }

    false
}

fn strip_wrapping_quotes(token: &str) -> &str {
    token.trim_matches(|c| c == '"' || c == '\'')
}

fn looks_like_path(candidate: &str) -> bool {
    candidate.starts_with('/')
        || candidate.starts_with("./")
        || candidate.starts_with("../")
        || candidate.starts_with('~')
        || candidate == "."
        || candidate == ".."
        || candidate.contains('/')
}

fn attached_short_option_value(token: &str) -> Option<&str> {
    // Examples:
    // -f/etc/passwd   -> /etc/passwd
    // -C../outside    -> ../outside
    // -I./include     -> ./include
    let body = token.strip_prefix('-')?;
    if body.starts_with('-') || body.len() < 2 {
        return None;
    }
    let value = body[1..].trim_start_matches('=').trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn redirection_target(token: &str) -> Option<&str> {
    let marker_idx = token.find(['<', '>'])?;
    let mut rest = &token[marker_idx + 1..];
    rest = rest.trim_start_matches(['<', '>']);
    rest = rest.trim_start_matches('&');
    rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn is_allowlist_entry_match(allowed: &str, executable: &str, executable_base: &str) -> bool {
    let allowed = strip_wrapping_quotes(allowed).trim();
    if allowed.is_empty() {
        return false;
    }

    // Explicit wildcard support for "allow any command name/path".
    if allowed == "*" {
        return true;
    }

    // Path-like allowlist entries must match the executable token exactly
    // after "~" expansion.
    if looks_like_path(allowed) {
        let allowed_path = expand_user_path(allowed);
        let executable_path = expand_user_path(executable);
        return executable_path == allowed_path;
    }

    // Command-name entries continue to match by basename.
    allowed == executable_base
}

impl SecurityPolicy {
    /// Canonical default allowed command list — Unix/macOS names and their
    /// Windows equivalents (.exe / .cmd / .bat) side by side so the same
    /// policy works cross-platform without user configuration.
    pub fn default_allowed_commands() -> Vec<String> {
        let cmds: &[&str] = &[
            // ── Version control ──
            "git", "gh", "hub", "glab",
            // Windows: git ships the same binary name on all platforms
            "git.exe",

            // ── JavaScript / Node ──
            "node", "npm", "npx", "yarn", "pnpm", "bun", "deno", "volta",
            "node.exe", "npm.cmd", "npx.cmd", "yarn.cmd", "pnpm.cmd",
            "bun.exe", "deno.exe",

            // ── Rust ──
            "cargo", "rustc", "rustup",
            "cargo.exe", "rustc.exe", "rustup.exe",

            // ── Python ──
            "python3", "python", "pip3", "pip",
            "python3.exe", "python.exe", "pip3.exe", "pip.exe",
            "uv", "rye", "pipx", "poetry", "conda", "mamba",
            "uv.exe", "pipx.exe", "poetry.exe",
            "pyenv",

            // ── Ruby ──
            "ruby", "gem", "bundle", "rake", "irb",
            "ruby.exe", "gem.cmd",
            "rbenv",

            // ── Go ──
            "go", "gofmt",
            "go.exe",

            // ── Java / JVM ──
            "java", "javac", "mvn", "gradle", "kotlin", "kotlinc",
            "java.exe", "javac.exe", "mvn.cmd", "gradle.bat",

            // ── .NET / C# ──
            "dotnet", "dotnet.exe",
            "nuget", "nuget.exe",

            // ── Swift ──
            "swift", "swiftc",

            // ── PHP ──
            "php", "composer",
            "php.exe", "composer.bat",

            // ── Lua ──
            "lua", "luajit",

            // ── R ──
            "Rscript", "R",

            // ── Julia ──
            "julia", "julia.exe",

            // ── Zig ──
            "zig", "zig.exe",

            // ── macOS package manager ──
            "brew",

            // ── Windows package managers & shells ──
            "winget", "choco", "scoop",
            "cmd", "cmd.exe",
            "powershell", "powershell.exe", "pwsh", "pwsh.exe",
            "wsl", "wsl.exe",
            "py", "py.exe",            // Python launcher for Windows

            // ── File system (Unix) ──
            "ls", "cat", "find", "grep", "echo", "pwd",
            "wc", "head", "tail", "touch", "mkdir", "cp", "mv", "rm",
            "chmod", "unzip", "tar", "ln", "stat", "realpath", "basename",
            "dirname", "readlink", "du", "df", "lsof",

            // ── File system (Windows) ──
            "dir", "type", "copy", "xcopy", "robocopy", "move", "del",
            "ren", "rename", "rd", "rmdir", "mklink",
            "attrib", "icacls",

            // ── Modern CLI replacements ──
            "rg", "fd", "bat", "eza", "exa", "delta", "hyperfine",
            "sd", "choose", "procs", "bottom", "btm", "dust", "duf",
            "tokei", "loc", "cloc",
            "rg.exe", "fd.exe", "bat.exe", "eza.exe",

            // ── Text processing (Unix) ──
            "sed", "awk", "sort", "cut", "tr", "tee", "xargs",
            "jq", "yq", "gron", "htmlq", "pup",
            "diff", "patch", "column", "fmt", "fold", "pr",
            "strings", "hexdump", "xxd", "od",

            // ── Text processing (Windows) ──
            "find.exe", "findstr", "sort.exe", "fc",

            // ── Network (Unix/cross-platform) ──
            "curl", "wget", "http", "httpie", "xh",
            "ping", "nslookup", "dig", "host", "whois",
            "traceroute", "tracert",
            "netstat", "ss", "ifconfig", "ipconfig",
            "ip", "route",
            "nc", "ncat",
            "ssh", "scp", "sftp", "rsync",

            // ── Network (Windows) ──
            "curl.exe", "wget.exe",
            "ping.exe", "tracert.exe", "netstat.exe",
            "ipconfig.exe", "nslookup.exe",

            // ── Encoding / hashing ──
            "base64", "md5", "md5sum", "sha1sum", "sha256sum", "shasum",
            "openssl",
            "base64.exe",

            // ── Compression ──
            "gzip", "gunzip", "bzip2", "bunzip2", "xz", "lzma",
            "zip", "7z", "7za",
            "7z.exe", "7za.exe",

            // ── System / utility (Unix) ──
            "date", "which", "whereis", "type", "command",
            "open", "xdg-open", "xclip", "xsel",
            "osascript", "sleep", "watch", "timeout",
            "ps", "top", "htop", "kill", "killall", "pkill", "pgrep",
            "uname", "hostname", "id", "whoami", "groups",
            "env", "printenv", "set",
            "file", "xattr",

            // ── System / utility (macOS-specific) ──
            "pbcopy", "pbpaste", "say", "afplay", "caffeinate",
            "launchctl", "plutil", "defaults", "scutil", "dscl",
            "networksetup", "system_profiler",
            "sips", "mdls", "mdfind", "spotlight",
            "ditto", "hdiutil",

            // ── System / utility (Windows-specific) ──
            "where", "where.exe",
            "tasklist", "taskkill", "taskkill.exe",
            "sc", "sc.exe",
            "reg", "reg.exe",
            "sfc", "sfc.exe",
            "cls", "ver",
            "systeminfo", "hostname.exe",
            "wmic", "wmic.exe",
            "msiexec", "msiexec.exe",
            "setx", "setx.exe",
            "net", "net.exe",
            "bcdedit",
            "shutdown.exe",   // Windows shutdown (different risk from Unix shutdown)

            // ── Build tools ──
            "make", "cmake", "ninja", "meson",
            "bazel", "buck", "ant",
            "make.exe", "cmake.exe", "ninja.exe",

            // ── Cloud & infra CLIs ──
            "docker", "docker-compose",
            "kubectl", "helm", "k9s", "skaffold",
            "terraform", "tofu",
            "ansible", "ansible-playbook",
            "vault",
            "aws", "az", "gcloud", "gsutil",
            "heroku", "railway", "fly",
            "vercel", "netlify", "wrangler",
            "docker.exe",

            // ── Databases ──
            "sqlite3", "psql", "mysql", "mysqldump",
            "redis-cli", "mongosh", "mongo",
            "sqlite3.exe",

            // ── Media / documents ──
            "ffmpeg", "ffprobe", "ffplay",
            "pandoc", "pdflatex", "xelatex", "lualatex",
            "convert",      // ImageMagick
            "magick",       // ImageMagick v7
            "tesseract",
            "exiftool",
            "yt-dlp", "youtube-dl",
            "libreoffice",
            "ffmpeg.exe",

            // ── Load / perf testing ──
            "ab", "wrk", "siege", "hey", "vegeta", "k6",

            // ── Linters / formatters ──
            "prettier", "eslint", "tsc",
            "black", "ruff", "flake8", "mypy", "pylint", "isort",
            "rubocop",
            "rustfmt", "clippy-driver",
            "shellcheck", "shfmt",
            "hadolint",
            "prettier.cmd", "eslint.cmd",
        ];

        cmds.iter().map(|s| s.to_string()).collect()
    }

    // ── Risk Classification ──────────────────────────────────────────────
    // Risk is assessed per-segment (split on shell operators), and the
    // highest risk across all segments wins. This prevents bypasses like
    // `ls && rm -rf /` from being classified as Low just because `ls` is safe.

    /// Classify command risk. Any high-risk segment marks the whole command high.
    pub fn command_risk_level(&self, command: &str) -> CommandRiskLevel {
        let mut saw_medium = false;

        for segment in split_unquoted_segments(command) {
            let cmd_part = skip_env_assignments(&segment);
            let mut words = cmd_part.split_whitespace();
            let Some(base_raw) = words.next() else {
                continue;
            };

            let base = base_raw
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();

            let args: Vec<String> = words.map(|w| w.to_ascii_lowercase()).collect();
            let joined_segment = cmd_part.to_ascii_lowercase();

            // High-risk: truly destructive / system-altering commands that
            // are permanently blocked unless the user explicitly removes them
            // from this list. Do NOT put everyday tools like curl/wget/ssh
            // here — they should be Medium so `approved=true` can unlock them.
            if matches!(
                base.as_str(),
                "mkfs"
                    | "dd"
                    | "shutdown"
                    | "reboot"
                    | "halt"
                    | "poweroff"
                    | "sudo"
                    | "su"
                    | "chown"
                    | "useradd"
                    | "userdel"
                    | "usermod"
                    | "passwd"
                    | "mount"
                    | "umount"
                    | "iptables"
                    | "ufw"
                    | "firewall-cmd"
                    | "rm"
                    | "chmod"
                    | "curl" | "curl.exe"
                    | "wget" | "wget.exe"
                    | "ssh" | "scp" | "sftp"
                    | "nc" | "ncat" | "netcat"
                    | "ftp" | "telnet"
            ) {
                return CommandRiskLevel::High;
            }

            if joined_segment.contains("rm -rf /")
                || joined_segment.contains("rm -fr /")
                || joined_segment.contains(":(){:|:&};:")
            {
                return CommandRiskLevel::High;
            }

            // Medium-risk commands (state-changing, but not inherently destructive)
            let medium = match base.as_str() {
                "git" => args.first().is_some_and(|verb| {
                    matches!(
                        verb.as_str(),
                        "commit"
                            | "push"
                            | "reset"
                            | "clean"
                            | "rebase"
                            | "merge"
                            | "cherry-pick"
                            | "revert"
                            | "branch"
                            | "checkout"
                            | "switch"
                            | "tag"
                    )
                }),
                "npm" | "npm.cmd" | "pnpm" | "pnpm.cmd" | "yarn" | "yarn.cmd" | "bun" => {
                    args.first().is_some_and(|verb| {
                        matches!(
                            verb.as_str(),
                            "install" | "add" | "remove" | "uninstall" | "update" | "publish"
                        )
                    })
                }
                "cargo" => args.first().is_some_and(|verb| {
                    matches!(
                        verb.as_str(),
                        "add" | "remove" | "install" | "clean" | "publish"
                    )
                }),
                // File system mutations
                "touch" | "mkdir" | "mv" | "cp" | "ln" => true,
                // File deletion (non-rm variants; rm is High)
                "del" | "rd" | "rmdir" => true,
                // Network — rsync kept Medium since it's commonly safe local use
                "rsync" => true,
                // Permission changes (icacls/attrib kept Medium; chmod is High)
                "icacls" | "attrib" => true,
                // Package managers that install system-wide software
                "pip" | "pip.exe" | "pip3" | "pip3.exe"
                | "brew"
                | "gem" | "gem.cmd"
                | "composer" | "composer.bat"
                | "conda" | "mamba"
                | "poetry" | "poetry.exe"
                | "uv" | "uv.exe"
                | "rye" | "pipx" | "pipx.exe"
                | "winget" | "choco" | "scoop"
                | "apt" | "apt-get" | "yum" | "dnf" | "pacman" | "zypper"
                | "nuget" | "nuget.exe" => true,
                // Cloud / infra CLIs that mutate remote state
                "docker" | "docker.exe" | "docker-compose"
                | "kubectl" | "helm"
                | "terraform" | "tofu"
                | "ansible" | "ansible-playbook"
                | "aws" | "az" | "gcloud" | "gsutil" => true,
                _ => false,
            };

            saw_medium |= medium;
        }

        if saw_medium {
            CommandRiskLevel::Medium
        } else {
            CommandRiskLevel::Low
        }
    }

    // ── Command Execution Policy Gate ──────────────────────────────────────
    // Validation follows a strict precedence order:
    //   1. Allowlist check (is the base command permitted at all?)
    //   2. Risk classification (high / medium / low)
    //   3. Policy flags (block_high_risk_commands, require_approval_for_medium_risk)
    //   4. Autonomy level × approval status (supervised requires explicit approval)
    // This ordering ensures deny-by-default: unknown commands are rejected
    // before any risk or autonomy logic runs.

    /// Validate full command execution policy (allowlist + risk gate).
    pub fn validate_command_execution(
        &self,
        command: &str,
        approved: bool,
    ) -> Result<CommandRiskLevel, String> {
        if !self.is_command_allowed(command) {
            return Err(format!("Command not allowed by security policy: {command}"));
        }

        let risk = self.command_risk_level(command);

        if risk == CommandRiskLevel::High {
            if self.blocks_high_risk_commands() {
                return Err("Command blocked: high-risk command is disallowed by policy".into());
            }
            if self.autonomy == AutonomyLevel::Supervised && !approved {
                return Err(
                    "Command requires explicit approval (approved=true): high-risk operation"
                        .into(),
                );
            }
        }

        if risk == CommandRiskLevel::Medium
            && self.autonomy == AutonomyLevel::Supervised
            && self.requires_medium_risk_approval()
            && !approved
        {
            return Err(
                "Command requires explicit approval (approved=true): medium-risk operation".into(),
            );
        }

        Ok(risk)
    }

    // ── Layered Command Allowlist ──────────────────────────────────────────
    // Defence-in-depth: five independent gates run in order before the
    // per-segment allowlist check. Each gate targets a specific bypass
    // technique. If any gate rejects, the whole command is blocked.

    /// Check if a shell command is allowed.
    ///
    /// Validates the **entire** command string, not just the first word:
    /// - Blocks subshell operators (`` ` ``, `$(`) that hide arbitrary execution
    /// - Splits on command separators (`|`, `&&`, `||`, `;`, newlines) and
    ///   validates each sub-command against the allowlist
    /// - Blocks single `&` background chaining (`&&` remains supported)
    /// - Blocks shell redirections (`<`, `>`, `>>`) that can bypass path policy
    /// - Blocks dangerous arguments (e.g. `find -exec`, `git config`)
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if self.autonomy == AutonomyLevel::ReadOnly {
            return false;
        }

        // Strip heredoc bodies before character-level checks so script content
        // (e.g. `>` in a JXA `if (x > 0)` expression) is not mistaken for a
        // shell redirection. `split_unquoted_segments` already handles heredocs
        // natively, so the original `command` is kept for that call.
        let stripped = strip_heredoc_bodies(command);
        let command = stripped.as_str();

        // Block subshell/expansion operators — these allow hiding arbitrary
        // commands inside an allowed command (e.g. `echo $(rm -rf /)`) and
        // bypassing path checks through variable indirection. The helper below
        // ignores escapes and literals inside single quotes, so `$(` or `${`
        // literals are permitted there.
        if command.contains('`')
            || contains_unquoted_shell_variable_expansion(command)
            || command.contains("<(")
            || command.contains(">(")
        {
            return false;
        }

        // Block shell redirections (`<`, `>`, `>>`) — they can read/write
        // arbitrary paths and bypass path checks.
        // Ignore quoted literals, e.g. `echo "a>b"` and `echo "a<b"`.
        if contains_unquoted_char(command, '>') || contains_unquoted_dangerous_lt(command) {
            return false;
        }


        // Block background command chaining (`&`), which can hide extra
        // sub-commands and outlive timeout expectations. Keep `&&` allowed.
        if contains_unquoted_single_ampersand(command) {
            return false;
        }

        // Split on unquoted command separators and validate each sub-command.
        let segments = split_unquoted_segments(command);
        for segment in &segments {
            // Strip leading env var assignments (e.g. FOO=bar cmd)
            let cmd_part = skip_env_assignments(segment);

            let mut words = cmd_part.split_whitespace();
            let executable = strip_wrapping_quotes(words.next().unwrap_or("")).trim();
            let base_cmd = executable.rsplit('/').next().unwrap_or("");

            if base_cmd.is_empty() {
                continue;
            }

            if !self
                .allowed_commands_snapshot()
                .iter()
                .any(|allowed| is_allowlist_entry_match(allowed, executable, base_cmd))
            {
                return false;
            }

            // Validate arguments for the command
            let args: Vec<String> = words.map(|w| w.to_string()).collect();
            if !self.is_args_safe(base_cmd, &args) {
                return false;
            }
        }

        // At least one command must be present
        let has_cmd = segments.iter().any(|s| {
            let s = skip_env_assignments(s.trim());
            s.split_whitespace().next().is_some_and(|w| !w.is_empty())
        });

        has_cmd
    }

    /// Check for dangerous arguments that allow sub-command execution.
    fn is_args_safe(&self, base: &str, args: &[String]) -> bool {
        let base = base.to_ascii_lowercase();
        match base.as_str() {
            "find" => {
                // find -exec and find -ok allow arbitrary command execution
                !args.iter().any(|arg| {
                    let normalized = arg.to_ascii_lowercase();
                    normalized == "-exec" || normalized == "-ok"
                })
            }
            "git" => {
                // git config, alias, and -c can be used to set dangerous options
                // (e.g. git config core.editor "rm -rf /")
                !args.iter().any(|arg| {
                    let normalized = arg.to_ascii_lowercase();
                    normalized == "config"
                        || normalized.starts_with("config.")
                        || normalized == "alias"
                        || normalized.starts_with("alias.")
                        || arg == "-c"
                })
            }
            _ => true,
        }
    }

    /// Return the first path-like argument blocked by path policy.
    ///
    /// This is best-effort token parsing for shell commands and is intended
    /// as a safety gate before command execution.
    pub fn forbidden_path_argument(&self, command: &str) -> Option<String> {
        let forbidden_candidate = |raw: &str| {
            let candidate = strip_wrapping_quotes(raw).trim();
            if candidate.is_empty() || candidate.contains("://") {
                return None;
            }
            if looks_like_path(candidate) && !self.is_path_allowed(candidate) {
                Some(candidate.to_string())
            } else {
                None
            }
        };

        for segment in split_unquoted_segments(command) {
            let cmd_part = skip_env_assignments(&segment);
            let mut words = cmd_part.split_whitespace();
            let Some(executable) = words.next() else {
                continue;
            };

            // Cover inline forms like `cat</etc/passwd`.
            if let Some(target) = redirection_target(strip_wrapping_quotes(executable)) {
                if let Some(blocked) = forbidden_candidate(target) {
                    return Some(blocked);
                }
            }

            for token in words {
                let candidate = strip_wrapping_quotes(token).trim();
                if candidate.is_empty() || candidate.contains("://") {
                    continue;
                }

                if let Some(target) = redirection_target(candidate) {
                    if let Some(blocked) = forbidden_candidate(target) {
                        return Some(blocked);
                    }
                }

                // Handle option assignment forms like `--file=/etc/passwd`.
                if candidate.starts_with('-') {
                    if let Some((_, value)) = candidate.split_once('=') {
                        if let Some(blocked) = forbidden_candidate(value) {
                            return Some(blocked);
                        }
                    }
                    if let Some(value) = attached_short_option_value(candidate) {
                        if let Some(blocked) = forbidden_candidate(value) {
                            return Some(blocked);
                        }
                    }
                    continue;
                }

                if let Some(blocked) = forbidden_candidate(candidate) {
                    return Some(blocked);
                }
            }
        }

        None
    }

    // ── Path Validation ────────────────────────────────────────────────
    // Layered checks: null-byte injection → component-level traversal →
    // URL-encoded traversal → tilde expansion → absolute-path block →
    // forbidden-prefix match. Each layer addresses a distinct escape
    // technique; together they enforce workspace confinement.

    /// Check if a file path is allowed (no path traversal, within workspace)
    pub fn is_path_allowed(&self, path: &str) -> bool {
        // Block null bytes (can truncate paths in C-backed syscalls)
        if path.contains('\0') {
            return false;
        }

        // Block path traversal: check for ".." as a path component
        if Path::new(path)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return false;
        }

        // Block URL-encoded traversal attempts (e.g. ..%2f)
        let lower = path.to_lowercase();
        if lower.contains("..%2f") || lower.contains("%2f..") {
            return false;
        }

        // Reject "~user" forms because the shell expands them at runtime and
        // they can escape workspace policy.
        if path.starts_with('~') && path != "~" && !path.starts_with("~/") {
            return false;
        }

        // Expand "~" for consistent matching with forbidden paths and allowlists.
        let expanded_path = expand_user_path(path);

        // Block absolute paths when workspace_only is set
        if self.workspace_only && expanded_path.is_absolute() {
            return false;
        }

        // Block forbidden paths using path-component-aware matching
        for forbidden in &self.forbidden_paths {
            let forbidden_path = expand_user_path(forbidden);
            if expanded_path.starts_with(forbidden_path) {
                return false;
            }
        }

        true
    }

    /// Validate that a resolved path is inside the workspace or an allowed root.
    /// Call this AFTER joining `workspace_dir` + relative path and canonicalizing.
    pub fn is_resolved_path_allowed(&self, resolved: &Path) -> bool {
        // Prefer canonical workspace root so `/a/../b` style config paths don't
        // cause false positives or negatives.
        let workspace_root = self
            .workspace_dir
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_dir.clone());
        if resolved.starts_with(&workspace_root) {
            return true;
        }

        // Check extra allowed roots (e.g. shared skills directories) before
        // forbidden checks so explicit allowlists can coexist with broad
        // default forbidden roots such as `/home` and `/tmp`.
        for root in &self.allowed_roots {
            let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
            if resolved.starts_with(&canonical) {
                return true;
            }
        }

        // For paths outside workspace/allowlist, block forbidden roots to
        // prevent symlink escapes and sensitive directory access.
        for forbidden in &self.forbidden_paths {
            let forbidden_path = expand_user_path(forbidden);
            if resolved.starts_with(&forbidden_path) {
                return false;
            }
        }

        // When workspace_only is disabled the user explicitly opted out of
        // workspace confinement after forbidden-path checks are applied.
        if !self.workspace_only {
            return true;
        }

        false
    }

    pub fn resolved_path_violation_message(&self, resolved: &Path) -> String {
        let guidance = if self.allowed_roots.is_empty() {
            "Add the directory to [autonomy].allowed_roots (for example: allowed_roots = [\"/absolute/path\"]), or move the file into the workspace."
        } else {
            "Add a matching parent directory to [autonomy].allowed_roots, or move the file into the workspace."
        };

        format!(
            "Resolved path escapes workspace allowlist: {}. {}",
            resolved.display(),
            guidance
        )
    }

    /// Check if autonomy level permits any action at all
    pub fn can_act(&self) -> bool {
        self.autonomy != AutonomyLevel::ReadOnly
    }

    // ── Tool Operation Gating ──────────────────────────────────────────────
    // Read operations bypass autonomy and rate checks because they have
    // no side effects. Act operations must pass both the autonomy gate
    // (not read-only) and the sliding-window rate limiter.

    /// Enforce policy for a tool operation.
    ///
    /// Read operations are always allowed by autonomy/rate gates.
    /// Act operations require non-readonly autonomy and available action budget.
    pub fn enforce_tool_operation(
        &self,
        operation: ToolOperation,
        operation_name: &str,
    ) -> Result<(), String> {
        match operation {
            ToolOperation::Read => Ok(()),
            ToolOperation::Act => {
                if !self.can_act() {
                    return Err(format!(
                        "Security policy: read-only mode, cannot perform '{operation_name}'"
                    ));
                }

                if !self.record_action() {
                    return Err("Rate limit exceeded: action budget exhausted".to_string());
                }

                Ok(())
            }
        }
    }

    /// Record an action and check if the rate limit has been exceeded.
    /// Returns `true` if the action is allowed, `false` if rate-limited.
    pub fn record_action(&self) -> bool {
        let live = self.live_limit.load(Ordering::Relaxed);
        let limit = if live == u32::MAX { self.max_actions_per_hour } else { live } as usize;
        let count = self.tracker.record();
        count <= limit
    }

    /// Check if the rate limit would be exceeded without recording.
    pub fn is_rate_limited(&self) -> bool {
        let live = self.live_limit.load(Ordering::Relaxed);
        let limit = if live == u32::MAX { self.max_actions_per_hour } else { live } as usize;
        self.tracker.count() >= limit
    }

    /// Update the per-hour action limit at runtime without restarting the daemon.
    /// Called by the config hot-reload path when `max_actions_per_hour` changes.
    pub fn set_max_actions_per_hour(&self, n: u32) {
        self.live_limit.store(n, Ordering::Relaxed);
    }

    /// Update the command allowlist at runtime without restarting the daemon.
    pub fn set_allowed_commands(&self, commands: Vec<String>) {
        let mut live_commands = self.live_allowed_commands.lock();
        *live_commands = commands;
        self.live_allowed_commands_overridden
            .store(true, Ordering::Relaxed);
    }

    /// Snapshot the current runtime allowlist.
    pub fn allowed_commands_snapshot(&self) -> Vec<String> {
        if self
            .live_allowed_commands_overridden
            .load(Ordering::Relaxed)
        {
            self.live_allowed_commands.lock().clone()
        } else {
            self.allowed_commands.clone()
        }
    }

    /// Check whether medium-risk commands currently require explicit approval.
    pub fn requires_medium_risk_approval(&self) -> bool {
        self.live_require_approval_for_medium_risk
            .load(Ordering::Relaxed)
    }

    /// Check whether high-risk commands are currently blocked outright.
    pub fn blocks_high_risk_commands(&self) -> bool {
        self.live_block_high_risk_commands.load(Ordering::Relaxed)
    }

    /// Update the command-risk policy at runtime without restarting the daemon.
    /// Called by the config hot-reload path when the approval flags change.
    pub fn set_command_risk_policy(
        &self,
        require_approval_for_medium_risk: bool,
        block_high_risk_commands: bool,
    ) {
        self.live_require_approval_for_medium_risk
            .store(require_approval_for_medium_risk, Ordering::Relaxed);
        self.live_block_high_risk_commands
            .store(block_high_risk_commands, Ordering::Relaxed);
    }

    /// Build from config sections
    pub fn from_config(
        autonomy_config: &crate::config::AutonomyConfig,
        workspace_dir: &Path,
    ) -> Self {
        Self {
            autonomy: autonomy_config.level,
            workspace_dir: workspace_dir.to_path_buf(),
            workspace_only: autonomy_config.workspace_only,
            allowed_commands: autonomy_config.allowed_commands.clone(),
            forbidden_paths: autonomy_config.forbidden_paths.clone(),
            allowed_roots: autonomy_config
                .allowed_roots
                .iter()
                .map(|root| {
                    let expanded = expand_user_path(root);
                    if expanded.is_absolute() {
                        expanded
                    } else {
                        workspace_dir.join(expanded)
                    }
                })
                .collect(),
            max_actions_per_hour: autonomy_config.max_actions_per_hour,
            max_cost_per_day_cents: autonomy_config.max_cost_per_day_cents,
            require_approval_for_medium_risk: autonomy_config.require_approval_for_medium_risk,
            block_high_risk_commands: autonomy_config.block_high_risk_commands,
            live_allowed_commands: Arc::new(Mutex::new(autonomy_config.allowed_commands.clone())),
            live_allowed_commands_overridden: Arc::new(AtomicBool::new(false)),
            live_require_approval_for_medium_risk: Arc::new(AtomicBool::new(
                autonomy_config.require_approval_for_medium_risk,
            )),
            live_block_high_risk_commands: Arc::new(AtomicBool::new(
                autonomy_config.block_high_risk_commands,
            )),
            shell_env_passthrough: autonomy_config.shell_env_passthrough.clone(),
            tracker: ActionTracker::new(),
            live_limit: Arc::new(AtomicU32::new(u32::MAX)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_policy() -> SecurityPolicy {
        SecurityPolicy::default()
    }

    fn readonly_policy() -> SecurityPolicy {
        SecurityPolicy {
            autonomy: AutonomyLevel::ReadOnly,
            ..SecurityPolicy::default()
        }
    }

    fn full_policy() -> SecurityPolicy {
        SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            ..SecurityPolicy::default()
        }
    }

    // ── AutonomyLevel ────────────────────────────────────────

    #[test]
    fn autonomy_default_is_supervised() {
        assert_eq!(AutonomyLevel::default(), AutonomyLevel::Supervised);
    }

    #[test]
    fn autonomy_serde_roundtrip() {
        let json = serde_json::to_string(&AutonomyLevel::Full).unwrap();
        assert_eq!(json, "\"full\"");
        let parsed: AutonomyLevel = serde_json::from_str("\"readonly\"").unwrap();
        assert_eq!(parsed, AutonomyLevel::ReadOnly);
        let parsed2: AutonomyLevel = serde_json::from_str("\"supervised\"").unwrap();
        assert_eq!(parsed2, AutonomyLevel::Supervised);
    }

    #[test]
    fn can_act_readonly_false() {
        assert!(!readonly_policy().can_act());
    }

    #[test]
    fn can_act_supervised_true() {
        assert!(default_policy().can_act());
    }

    #[test]
    fn can_act_full_true() {
        assert!(full_policy().can_act());
    }

    #[test]
    fn enforce_tool_operation_read_allowed_in_readonly_mode() {
        let p = readonly_policy();
        assert!(p
            .enforce_tool_operation(ToolOperation::Read, "memory_recall")
            .is_ok());
    }

    #[test]
    fn enforce_tool_operation_act_blocked_in_readonly_mode() {
        let p = readonly_policy();
        let err = p
            .enforce_tool_operation(ToolOperation::Act, "memory_store")
            .unwrap_err();
        assert!(err.contains("read-only mode"));
    }

    #[test]
    fn enforce_tool_operation_act_uses_rate_budget() {
        let p = SecurityPolicy {
            max_actions_per_hour: 0,
            ..default_policy()
        };
        let err = p
            .enforce_tool_operation(ToolOperation::Act, "memory_store")
            .unwrap_err();
        assert!(err.contains("Rate limit exceeded"));
    }

    #[test]
    fn runtime_command_risk_policy_can_be_updated() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            allowed_commands: vec!["rm".into()],
            block_high_risk_commands: true,
            ..SecurityPolicy::default()
        };

        assert!(p.blocks_high_risk_commands());
        assert!(p.validate_command_execution("rm file.txt", false).is_err());

        p.set_command_risk_policy(false, false);

        assert!(!p.blocks_high_risk_commands());
        assert!(!p.requires_medium_risk_approval());
        let result = p.validate_command_execution("rm file.txt", false);
        assert_eq!(result.unwrap(), CommandRiskLevel::High);
    }

    #[test]
    fn runtime_medium_risk_policy_can_be_relaxed() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            allowed_commands: vec!["touch".into()],
            require_approval_for_medium_risk: true,
            ..SecurityPolicy::default()
        };

        assert!(p.requires_medium_risk_approval());
        assert!(p.validate_command_execution("touch test.txt", false).is_err());

        p.set_command_risk_policy(false, p.blocks_high_risk_commands());

        assert!(!p.requires_medium_risk_approval());
        let result = p.validate_command_execution("touch test.txt", false);
        assert_eq!(result.unwrap(), CommandRiskLevel::Medium);
    }

    #[test]
    fn runtime_allowlist_can_be_updated() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            allowed_commands: vec!["echo".into()],
            ..SecurityPolicy::default()
        };

        assert!(p.is_command_allowed("echo hello"));
        assert!(!p.is_command_allowed("git status"));

        p.set_allowed_commands(vec!["git".into(), "echo".into()]);

        let snapshot = p.allowed_commands_snapshot();
        assert!(snapshot.contains(&"git".to_string()));
        assert!(p.is_command_allowed("git status"));
    }

    // ── is_command_allowed ───────────────────────────────────

    #[test]
    fn allowed_commands_basic() {
        let p = default_policy();
        assert!(p.is_command_allowed("ls"));
        assert!(p.is_command_allowed("git status"));
        assert!(p.is_command_allowed("cargo build --release"));
        assert!(p.is_command_allowed("cat file.txt"));
        assert!(p.is_command_allowed("grep -r pattern ."));
        assert!(p.is_command_allowed("date"));
    }

    #[test]
    fn blocked_commands_basic() {
        let p = default_policy();
        assert!(!p.is_command_allowed("rm -rf /"));
        assert!(!p.is_command_allowed("sudo apt install"));
        assert!(!p.is_command_allowed("curl http://evil.com"));
        assert!(!p.is_command_allowed("wget http://evil.com"));
        assert!(!p.is_command_allowed("python3 exploit.py"));
        assert!(!p.is_command_allowed("node malicious.js"));
    }

    #[test]
    fn readonly_blocks_all_commands() {
        let p = readonly_policy();
        assert!(!p.is_command_allowed("ls"));
        assert!(!p.is_command_allowed("cat file.txt"));
        assert!(!p.is_command_allowed("echo hello"));
    }

    #[test]
    fn full_autonomy_still_uses_allowlist() {
        let p = full_policy();
        assert!(p.is_command_allowed("ls"));
        assert!(!p.is_command_allowed("rm -rf /"));
    }

    #[test]
    fn command_with_absolute_path_extracts_basename() {
        let p = default_policy();
        assert!(p.is_command_allowed("/usr/bin/git status"));
        assert!(p.is_command_allowed("/bin/ls -la"));
    }

    #[test]
    fn allowlist_supports_explicit_executable_paths() {
        let p = SecurityPolicy {
            allowed_commands: vec!["/usr/bin/antigravity".into()],
            ..SecurityPolicy::default()
        };

        assert!(p.is_command_allowed("/usr/bin/antigravity"));
        assert!(!p.is_command_allowed("antigravity"));
    }

    #[test]
    fn allowlist_supports_wildcard_entry() {
        let p = SecurityPolicy {
            allowed_commands: vec!["*".into()],
            ..SecurityPolicy::default()
        };

        assert!(p.is_command_allowed("python3 --version"));
        assert!(p.is_command_allowed("/usr/bin/antigravity"));

        // Wildcard still respects risk gates in validate_command_execution.
        let blocked = p.validate_command_execution("rm -rf /tmp/test", true);
        assert!(blocked.is_err());
        assert!(blocked.unwrap_err().contains("high-risk"));
    }

    #[test]
    fn empty_command_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed(""));
        assert!(!p.is_command_allowed("   "));
    }

    #[test]
    fn command_with_pipes_validates_all_segments() {
        let p = default_policy();
        // Both sides of the pipe are in the allowlist
        assert!(p.is_command_allowed("ls | grep foo"));
        assert!(p.is_command_allowed("cat file.txt | wc -l"));
        // Second command not in allowlist — blocked
        assert!(!p.is_command_allowed("ls | curl http://evil.com"));
        assert!(!p.is_command_allowed("echo hello | python3 -"));
    }

    #[test]
    fn custom_allowlist() {
        let p = SecurityPolicy {
            allowed_commands: vec!["docker".into(), "kubectl".into()],
            ..SecurityPolicy::default()
        };
        assert!(p.is_command_allowed("docker ps"));
        assert!(p.is_command_allowed("kubectl get pods"));
        assert!(!p.is_command_allowed("ls"));
        assert!(!p.is_command_allowed("git status"));
    }

    #[test]
    fn empty_allowlist_blocks_everything() {
        let p = SecurityPolicy {
            allowed_commands: vec![],
            ..SecurityPolicy::default()
        };
        assert!(!p.is_command_allowed("ls"));
        assert!(!p.is_command_allowed("echo hello"));
    }

    #[test]
    fn command_risk_low_for_read_commands() {
        let p = default_policy();
        assert_eq!(p.command_risk_level("git status"), CommandRiskLevel::Low);
        assert_eq!(p.command_risk_level("ls -la"), CommandRiskLevel::Low);
    }

    #[test]
    fn command_risk_medium_for_mutating_commands() {
        let p = SecurityPolicy {
            allowed_commands: vec!["git".into(), "touch".into()],
            ..SecurityPolicy::default()
        };
        assert_eq!(
            p.command_risk_level("git reset --hard HEAD~1"),
            CommandRiskLevel::Medium
        );
        assert_eq!(
            p.command_risk_level("touch file.txt"),
            CommandRiskLevel::Medium
        );
    }

    #[test]
    fn command_risk_high_for_dangerous_commands() {
        let p = SecurityPolicy {
            allowed_commands: vec!["rm".into()],
            ..SecurityPolicy::default()
        };
        assert_eq!(
            p.command_risk_level("rm -rf /tmp/test"),
            CommandRiskLevel::High
        );
    }

    #[test]
    fn validate_command_requires_approval_for_medium_risk() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            require_approval_for_medium_risk: true,
            allowed_commands: vec!["touch".into()],
            ..SecurityPolicy::default()
        };

        let denied = p.validate_command_execution("touch test.txt", false);
        assert!(denied.is_err());
        assert!(denied.unwrap_err().contains("requires explicit approval"),);

        let allowed = p.validate_command_execution("touch test.txt", true);
        assert_eq!(allowed.unwrap(), CommandRiskLevel::Medium);
    }

    #[test]
    fn validate_command_blocks_high_risk_by_default() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            allowed_commands: vec!["rm".into()],
            ..SecurityPolicy::default()
        };

        let result = p.validate_command_execution("rm -rf /tmp/test", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("high-risk"));
    }

    #[test]
    fn validate_command_full_mode_skips_medium_risk_approval_gate() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            require_approval_for_medium_risk: true,
            allowed_commands: vec!["touch".into()],
            ..SecurityPolicy::default()
        };

        let result = p.validate_command_execution("touch test.txt", false);
        assert_eq!(result.unwrap(), CommandRiskLevel::Medium);
    }

    #[test]
    fn validate_command_rejects_background_chain_bypass() {
        let p = default_policy();
        let result = p.validate_command_execution("ls & python3 -c 'print(1)'", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    // ── is_path_allowed ─────────────────────────────────────

    #[test]
    fn relative_paths_allowed() {
        let p = default_policy();
        assert!(p.is_path_allowed("file.txt"));
        assert!(p.is_path_allowed("src/main.rs"));
        assert!(p.is_path_allowed("deep/nested/dir/file.txt"));
    }

    #[test]
    fn path_traversal_blocked() {
        let p = default_policy();
        assert!(!p.is_path_allowed("../etc/passwd"));
        assert!(!p.is_path_allowed("../../root/.ssh/id_rsa"));
        assert!(!p.is_path_allowed("foo/../../../etc/shadow"));
        assert!(!p.is_path_allowed(".."));
    }

    #[test]
    fn absolute_paths_blocked_when_workspace_only() {
        let p = default_policy();
        assert!(!p.is_path_allowed("/etc/passwd"));
        assert!(!p.is_path_allowed("/root/.ssh/id_rsa"));
        assert!(!p.is_path_allowed("/tmp/file.txt"));
    }

    #[test]
    fn absolute_paths_allowed_when_not_workspace_only() {
        let p = SecurityPolicy {
            workspace_only: false,
            forbidden_paths: vec![],
            ..SecurityPolicy::default()
        };
        assert!(p.is_path_allowed("/tmp/file.txt"));
    }

    #[test]
    fn forbidden_paths_blocked() {
        let p = SecurityPolicy {
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_path_allowed("/etc/passwd"));
        assert!(!p.is_path_allowed("/root/.bashrc"));
        assert!(!p.is_path_allowed("~/.ssh/id_rsa"));
        assert!(!p.is_path_allowed("~/.gnupg/pubring.kbx"));
    }

    #[test]
    fn empty_path_allowed() {
        let p = default_policy();
        assert!(p.is_path_allowed(""));
    }

    #[test]
    fn dotfile_in_workspace_allowed() {
        let p = default_policy();
        assert!(p.is_path_allowed(".gitignore"));
        assert!(p.is_path_allowed(".env"));
    }

    // ── from_config ─────────────────────────────────────────

    #[test]
    fn from_config_maps_all_fields() {
        let autonomy_config = crate::config::AutonomyConfig {
            level: AutonomyLevel::Full,
            workspace_only: false,
            allowed_commands: vec!["docker".into()],
            forbidden_paths: vec!["/secret".into()],
            max_actions_per_hour: 100,
            max_cost_per_day_cents: 1000,
            require_approval_for_medium_risk: false,
            block_high_risk_commands: false,
            shell_env_passthrough: vec!["DATABASE_URL".into()],
            ..crate::config::AutonomyConfig::default()
        };
        let workspace = PathBuf::from("/tmp/test-workspace");
        let policy = SecurityPolicy::from_config(&autonomy_config, &workspace);

        assert_eq!(policy.autonomy, AutonomyLevel::Full);
        assert!(!policy.workspace_only);
        assert_eq!(policy.allowed_commands, vec!["docker"]);
        assert_eq!(policy.forbidden_paths, vec!["/secret"]);
        assert_eq!(policy.max_actions_per_hour, 100);
        assert_eq!(policy.max_cost_per_day_cents, 1000);
        assert!(!policy.require_approval_for_medium_risk);
        assert!(!policy.block_high_risk_commands);
        assert_eq!(policy.shell_env_passthrough, vec!["DATABASE_URL"]);
        assert_eq!(policy.workspace_dir, PathBuf::from("/tmp/test-workspace"));
    }

    #[test]
    fn from_config_normalizes_allowed_roots() {
        let autonomy_config = crate::config::AutonomyConfig {
            allowed_roots: vec!["~/Desktop".into(), "shared-data".into()],
            ..crate::config::AutonomyConfig::default()
        };
        let workspace = PathBuf::from("/tmp/test-workspace");
        let policy = SecurityPolicy::from_config(&autonomy_config, &workspace);

        let expected_home_root = if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join("Desktop")
        } else {
            PathBuf::from("~/Desktop")
        };

        assert_eq!(policy.allowed_roots[0], expected_home_root);
        assert_eq!(policy.allowed_roots[1], workspace.join("shared-data"));
    }

    #[test]
    fn resolved_path_violation_message_includes_allowed_roots_guidance() {
        let p = default_policy();
        let msg = p.resolved_path_violation_message(Path::new("/tmp/outside.txt"));
        assert!(msg.contains("escapes workspace"));
        assert!(msg.contains("allowed_roots"));
    }

    // ── Default policy ──────────────────────────────────────

    #[test]
    fn default_policy_has_sane_values() {
        let p = SecurityPolicy::default();
        assert_eq!(p.autonomy, AutonomyLevel::Supervised);
        assert!(p.workspace_only);
        assert!(!p.allowed_commands.is_empty());
        assert!(!p.forbidden_paths.is_empty());
        assert!(p.max_actions_per_hour > 0);
        assert!(p.max_cost_per_day_cents > 0);
        assert!(p.require_approval_for_medium_risk);
        assert!(p.block_high_risk_commands);
        assert!(p.shell_env_passthrough.is_empty());
    }

    // ── ActionTracker / rate limiting ───────────────────────

    #[test]
    fn action_tracker_starts_at_zero() {
        let tracker = ActionTracker::new();
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn action_tracker_records_actions() {
        let tracker = ActionTracker::new();
        assert_eq!(tracker.record(), 1);
        assert_eq!(tracker.record(), 2);
        assert_eq!(tracker.record(), 3);
        assert_eq!(tracker.count(), 3);
    }

    #[test]
    fn record_action_allows_within_limit() {
        let p = SecurityPolicy {
            max_actions_per_hour: 5,
            ..SecurityPolicy::default()
        };
        for _ in 0..5 {
            assert!(p.record_action(), "should allow actions within limit");
        }
    }

    #[test]
    fn record_action_blocks_over_limit() {
        let p = SecurityPolicy {
            max_actions_per_hour: 3,
            ..SecurityPolicy::default()
        };
        assert!(p.record_action()); // 1
        assert!(p.record_action()); // 2
        assert!(p.record_action()); // 3
        assert!(!p.record_action()); // 4 — over limit
    }

    #[test]
    fn is_rate_limited_reflects_count() {
        let p = SecurityPolicy {
            max_actions_per_hour: 2,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_rate_limited());
        p.record_action();
        assert!(!p.is_rate_limited());
        p.record_action();
        assert!(p.is_rate_limited());
    }

    #[test]
    fn action_tracker_clone_is_independent() {
        let tracker = ActionTracker::new();
        tracker.record();
        tracker.record();
        let cloned = tracker.clone();
        assert_eq!(cloned.count(), 2);
        tracker.record();
        assert_eq!(tracker.count(), 3);
        assert_eq!(cloned.count(), 2); // clone is independent
    }

    // ── Edge cases: command injection ────────────────────────

    #[test]
    fn command_injection_semicolon_blocked() {
        let p = default_policy();
        // First word is "ls;" (with semicolon) — doesn't match "ls" in allowlist.
        // This is a safe default: chained commands are blocked.
        assert!(!p.is_command_allowed("ls; rm -rf /"));
    }

    #[test]
    fn command_injection_semicolon_no_space() {
        let p = default_policy();
        assert!(!p.is_command_allowed("ls;rm -rf /"));
    }

    #[test]
    fn quoted_semicolons_do_not_split_sqlite_command() {
        let p = SecurityPolicy {
            allowed_commands: vec!["sqlite3".into()],
            ..SecurityPolicy::default()
        };
        assert!(p.is_command_allowed(
            "sqlite3 /tmp/test.db \"CREATE TABLE t(id INT); INSERT INTO t VALUES(1); SELECT * FROM t;\""
        ));
        assert_eq!(
            p.command_risk_level(
                "sqlite3 /tmp/test.db \"CREATE TABLE t(id INT); INSERT INTO t VALUES(1); SELECT * FROM t;\""
            ),
            CommandRiskLevel::Low
        );
    }

    #[test]
    fn unquoted_semicolon_after_quoted_sql_still_splits_commands() {
        let p = SecurityPolicy {
            allowed_commands: vec!["sqlite3".into()],
            ..SecurityPolicy::default()
        };
        assert!(!p.is_command_allowed("sqlite3 /tmp/test.db \"SELECT 1;\"; rm -rf /"));
    }

    #[test]
    fn command_injection_backtick_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo `whoami`"));
        assert!(!p.is_command_allowed("echo `rm -rf /`"));
    }

    #[test]
    fn command_injection_dollar_paren_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo $(cat /etc/passwd)"));
        assert!(!p.is_command_allowed("echo $(rm -rf /)"));
    }

    #[test]
    fn command_injection_dollar_paren_literal_inside_single_quotes_allowed() {
        let p = default_policy();
        assert!(p.is_command_allowed("echo '$(cat /etc/passwd)'"));
    }

    #[test]
    fn command_injection_dollar_brace_literal_inside_single_quotes_allowed() {
        let p = default_policy();
        assert!(p.is_command_allowed("echo '${HOME}'"));
    }

    #[test]
    fn command_injection_dollar_brace_unquoted_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo ${HOME}"));
    }

    #[test]
    fn command_with_env_var_prefix() {
        let p = default_policy();
        // "FOO=bar" is the first word — not in allowlist
        assert!(!p.is_command_allowed("FOO=bar rm -rf /"));
    }

    #[test]
    fn command_newline_injection_blocked() {
        let p = default_policy();
        // Newline splits into two commands; "rm" is not in allowlist
        assert!(!p.is_command_allowed("ls\nrm -rf /"));
        // Both allowed — OK
        assert!(p.is_command_allowed("ls\necho hello"));
    }

    #[test]
    fn command_injection_and_chain_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("ls && rm -rf /"));
        assert!(!p.is_command_allowed("echo ok && curl http://evil.com"));
        // Both allowed — OK
        assert!(p.is_command_allowed("ls && echo done"));
    }

    #[test]
    fn command_injection_or_chain_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("ls || rm -rf /"));
        // Both allowed — OK
        assert!(p.is_command_allowed("ls || echo fallback"));
    }

    #[test]
    fn command_injection_background_chain_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("ls & rm -rf /"));
        assert!(!p.is_command_allowed("ls&rm -rf /"));
        assert!(!p.is_command_allowed("echo ok & python3 -c 'print(1)'"));
    }

    #[test]
    fn command_injection_redirect_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo secret > /etc/crontab"));
        assert!(!p.is_command_allowed("ls >> /tmp/exfil.txt"));
        assert!(!p.is_command_allowed("cat </etc/passwd"));
        assert!(!p.is_command_allowed("cat</etc/passwd"));
    }

    #[test]
    fn quoted_ampersand_and_redirect_literals_are_not_treated_as_operators() {
        let p = default_policy();
        assert!(p.is_command_allowed("echo \"A&B\""));
        assert!(p.is_command_allowed("echo \"A>B\""));
        assert!(p.is_command_allowed("echo \"A<B\""));
    }

    #[test]
    fn command_argument_injection_blocked() {
        let p = default_policy();
        // find -exec is a common bypass
        assert!(!p.is_command_allowed("find . -exec rm -rf {} +"));
        assert!(!p.is_command_allowed("find / -ok cat {} \\;"));
        // git config/alias can execute commands
        assert!(!p.is_command_allowed("git config core.editor \"rm -rf /\""));
        assert!(!p.is_command_allowed("git alias.st status"));
        assert!(!p.is_command_allowed("git -c core.editor=calc.exe commit"));
        // Legitimate commands should still work
        assert!(p.is_command_allowed("find . -name '*.txt'"));
        assert!(p.is_command_allowed("git status"));
        assert!(p.is_command_allowed("git add ."));
        assert!(p.is_command_allowed("git -C /tmp/repo status"));
    }

    #[test]
    fn git_uppercase_c_flag_remains_allowed() {
        let p = default_policy();
        assert!(p.is_command_allowed("git -C /Users/ibz/Desktop/reademy status"));
    }

    #[test]
    fn command_injection_dollar_brace_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo ${IFS}cat${IFS}/etc/passwd"));
    }

    #[test]
    fn command_injection_plain_dollar_var_blocked() {
        let p = default_policy();
        assert!(
            p.forbidden_path_argument("cat $HOME/.ssh/id_rsa").is_some(),
            "$HOME must expand so forbidden_paths still applies"
        );
        assert!(p.is_command_allowed("cat $HOME/.ssh/id_rsa"));
        assert!(!p.is_command_allowed("cat $SECRET_FILE"));
    }

    #[test]
    fn home_downloads_shell_pipeline_allowed_under_wildcard_allowlist() {
        let p = SecurityPolicy {
            allowed_commands: vec!["*".into()],
            workspace_only: false,
            // Narrow forbidden list so `$HOME/...` resolves under Linux `/home/...` in CI.
            forbidden_paths: vec!["~/.ssh".into()],
            ..SecurityPolicy::default()
        };
        let cmd = r#"mkdir -p "$HOME/Downloads/_gloamy_sort" && ls -1 "$HOME/Downloads""#;
        assert!(p.is_command_allowed(cmd));
        assert_eq!(p.forbidden_path_argument(cmd), None);
    }

    #[test]
    fn quoted_heredoc_opener_allows_double_angle() {
        let p = SecurityPolicy {
            allowed_commands: vec!["python3".into()],
            ..SecurityPolicy::default()
        };
        assert!(p.is_command_allowed("python3 - <<'PY'"));
        assert!(p.is_command_allowed("python3 - <<-'PY'"));
        assert!(p.is_command_allowed("python3 - <<\"PY\""));
    }

    #[test]
    fn multiline_quoted_heredoc_body_is_not_split_into_commands() {
        let p = SecurityPolicy {
            allowed_commands: vec!["python3".into(), "ls".into()],
            ..SecurityPolicy::default()
        };

        let cmd = "python3 - <<'PY'\nfrom pathlib import Path\nprint('ok')\nPY\nls";
        let segments = split_unquoted_segments(cmd);

        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            "python3 - <<'PY'\nfrom pathlib import Path\nprint('ok')\nPY"
        );
        assert_eq!(segments[1], "ls");
        assert!(p.is_command_allowed("python3 - <<'PY'\nprint('ok')\nPY"));
    }

    #[test]
    fn unquoted_heredoc_delimiter_still_blocks_lt_gate() {
        let p = SecurityPolicy {
            allowed_commands: vec!["python3".into()],
            ..SecurityPolicy::default()
        };
        assert!(!p.is_command_allowed("python3 - <<PY"));
    }

    #[test]
    fn jxa_heredoc_body_with_gt_and_semicolons_is_not_blocked() {
        let p = SecurityPolicy {
            allowed_commands: vec!["osascript".into(), "sleep".into()],
            ..SecurityPolicy::default()
        };
        let cmd = "osascript -e 'tell application \"Microsoft PowerPoint\" to activate' \
                   && sleep 2 \
                   && osascript -l JavaScript <<'JXA'\n\
                   const ppt = Application('Microsoft PowerPoint');\n\
                   try {\n\
                     if (ppt.presentations.length > 0) { ppt.activate(); }\n\
                   } catch (e) {}\n\
                   JXA";
        assert!(
            p.is_command_allowed(cmd),
            "JXA heredoc with > and ; in body should not be blocked by redirect gate"
        );
    }

    #[test]
    fn command_injection_tee_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("echo secret | tee /etc/crontab"));
        assert!(!p.is_command_allowed("ls | /usr/bin/tee outfile"));
        assert!(!p.is_command_allowed("tee file.txt"));
    }

    #[test]
    fn command_injection_process_substitution_blocked() {
        let p = default_policy();
        assert!(!p.is_command_allowed("cat <(echo pwned)"));
        assert!(!p.is_command_allowed("ls >(cat /etc/passwd)"));
    }

    #[test]
    fn command_env_var_prefix_with_allowed_cmd() {
        let p = default_policy();
        // env assignment + allowed command — OK
        assert!(p.is_command_allowed("FOO=bar ls"));
        assert!(p.is_command_allowed("LANG=C grep pattern file"));
        // env assignment + disallowed command — blocked
        assert!(!p.is_command_allowed("FOO=bar rm -rf /"));
    }

    #[test]
    fn forbidden_path_argument_detects_absolute_path() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("cat /etc/passwd"),
            Some("/etc/passwd".into())
        );
    }

    #[test]
    fn forbidden_path_argument_detects_parent_dir_reference() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("cat ../secret.txt"),
            Some("../secret.txt".into())
        );
        assert_eq!(
            p.forbidden_path_argument("find .. -name '*.rs'"),
            Some("..".into())
        );
    }

    #[test]
    fn forbidden_path_argument_allows_workspace_relative_paths() {
        let p = default_policy();
        assert_eq!(p.forbidden_path_argument("cat src/main.rs"), None);
        assert_eq!(p.forbidden_path_argument("grep -r todo ./src"), None);
    }

    #[test]
    fn forbidden_path_argument_detects_option_assignment_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("grep --file=/etc/passwd root ./src"),
            Some("/etc/passwd".into())
        );
        assert_eq!(
            p.forbidden_path_argument("cat --input=../secret.txt"),
            Some("../secret.txt".into())
        );
    }

    #[test]
    fn forbidden_path_argument_allows_safe_option_assignment_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("grep --file=./patterns.txt root ./src"),
            None
        );
    }

    #[test]
    fn forbidden_path_argument_detects_short_option_attached_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("grep -f/etc/passwd root ./src"),
            Some("/etc/passwd".into())
        );
        assert_eq!(
            p.forbidden_path_argument("git -C../outside status"),
            Some("../outside".into())
        );
    }

    #[test]
    fn forbidden_path_argument_allows_safe_short_option_attached_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("grep -f./patterns.txt root ./src"),
            None
        );
        assert_eq!(p.forbidden_path_argument("git -C./repo status"), None);
    }

    #[test]
    fn forbidden_path_argument_detects_tilde_user_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("cat ~root/.ssh/id_rsa"),
            Some("~root/.ssh/id_rsa".into())
        );
        assert_eq!(
            p.forbidden_path_argument("ls ~nobody"),
            Some("~nobody".into())
        );
    }

    #[test]
    fn forbidden_path_argument_detects_input_redirection_paths() {
        let p = default_policy();
        assert_eq!(
            p.forbidden_path_argument("cat </etc/passwd"),
            Some("/etc/passwd".into())
        );
        assert_eq!(
            p.forbidden_path_argument("cat</etc/passwd"),
            Some("/etc/passwd".into())
        );
    }

    // ── Edge cases: path traversal ──────────────────────────

    #[test]
    fn path_traversal_encoded_dots() {
        let p = default_policy();
        // Literal ".." in path — always blocked
        assert!(!p.is_path_allowed("foo/..%2f..%2fetc/passwd"));
    }

    #[test]
    fn path_traversal_double_dot_in_filename() {
        let p = default_policy();
        // ".." in a filename (not a path component) is allowed
        assert!(p.is_path_allowed("my..file.txt"));
        // But actual traversal components are still blocked
        assert!(!p.is_path_allowed("../etc/passwd"));
        assert!(!p.is_path_allowed("foo/../etc/passwd"));
    }

    #[test]
    fn path_with_null_byte_blocked() {
        let p = default_policy();
        assert!(!p.is_path_allowed("file\0.txt"));
    }

    #[test]
    fn path_symlink_style_absolute() {
        let p = default_policy();
        assert!(!p.is_path_allowed("/proc/self/root/etc/passwd"));
    }

    #[test]
    fn path_home_tilde_ssh() {
        let p = SecurityPolicy {
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_path_allowed("~/.ssh/id_rsa"));
        assert!(!p.is_path_allowed("~/.gnupg/secring.gpg"));
        assert!(!p.is_path_allowed("~root/.ssh/id_rsa"));
        assert!(!p.is_path_allowed("~nobody"));
    }

    #[test]
    fn path_var_run_blocked() {
        let p = SecurityPolicy {
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_path_allowed("/var/run/docker.sock"));
    }

    // ── Edge cases: rate limiter boundary ────────────────────

    #[test]
    fn rate_limit_exactly_at_boundary() {
        let p = SecurityPolicy {
            max_actions_per_hour: 1,
            ..SecurityPolicy::default()
        };
        assert!(p.record_action()); // 1 — exactly at limit
        assert!(!p.record_action()); // 2 — over
        assert!(!p.record_action()); // 3 — still over
    }

    #[test]
    fn rate_limit_zero_blocks_everything() {
        let p = SecurityPolicy {
            max_actions_per_hour: 0,
            ..SecurityPolicy::default()
        };
        assert!(!p.record_action());
    }

    #[test]
    fn rate_limit_high_allows_many() {
        let p = SecurityPolicy {
            max_actions_per_hour: 10000,
            ..SecurityPolicy::default()
        };
        for _ in 0..100 {
            assert!(p.record_action());
        }
    }

    // ── Edge cases: autonomy + command combos ────────────────

    #[test]
    fn readonly_blocks_even_safe_commands() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::ReadOnly,
            allowed_commands: vec!["ls".into(), "cat".into()],
            ..SecurityPolicy::default()
        };
        assert!(!p.is_command_allowed("ls"));
        assert!(!p.is_command_allowed("cat"));
        assert!(!p.can_act());
    }

    #[test]
    fn supervised_allows_listed_commands() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            allowed_commands: vec!["git".into()],
            ..SecurityPolicy::default()
        };
        assert!(p.is_command_allowed("git status"));
        assert!(!p.is_command_allowed("docker ps"));
    }

    #[test]
    fn full_autonomy_still_respects_forbidden_paths() {
        let p = SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_path_allowed("/etc/shadow"));
        assert!(!p.is_path_allowed("/root/.bashrc"));
    }

    #[test]
    fn workspace_only_false_allows_resolved_outside_workspace() {
        let workspace = std::env::temp_dir().join("gloamy_test_ws_only_false");
        let _ = std::fs::create_dir_all(&workspace);
        let canonical_workspace = workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.clone());

        let p = SecurityPolicy {
            workspace_dir: canonical_workspace.clone(),
            workspace_only: false,
            forbidden_paths: vec!["/etc".into(), "/var".into()],
            ..SecurityPolicy::default()
        };

        // Path outside workspace should be allowed when workspace_only=false
        let outside = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home"))
            .join("gloamy_outside_ws");
        assert!(
            p.is_resolved_path_allowed(&outside),
            "workspace_only=false must allow resolved paths outside workspace"
        );

        // Forbidden paths must still be blocked even with workspace_only=false
        assert!(
            !p.is_resolved_path_allowed(Path::new("/etc/passwd")),
            "forbidden paths must be blocked even when workspace_only=false"
        );
        assert!(
            !p.is_resolved_path_allowed(Path::new("/var/run/docker.sock")),
            "forbidden /var must be blocked even when workspace_only=false"
        );

        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn workspace_only_true_blocks_resolved_outside_workspace() {
        let workspace = std::env::temp_dir().join("gloamy_test_ws_only_true");
        let _ = std::fs::create_dir_all(&workspace);
        let canonical_workspace = workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.clone());

        let p = SecurityPolicy {
            workspace_dir: canonical_workspace.clone(),
            workspace_only: true,
            ..SecurityPolicy::default()
        };

        // Path inside workspace — allowed
        let inside = canonical_workspace.join("subdir");
        assert!(
            p.is_resolved_path_allowed(&inside),
            "path inside workspace must be allowed"
        );

        // Path outside workspace — blocked
        let outside = std::env::temp_dir()
            .canonicalize()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("gloamy_outside_ws_true");
        assert!(
            !p.is_resolved_path_allowed(&outside),
            "workspace_only=true must block resolved paths outside workspace"
        );

        let _ = std::fs::remove_dir_all(&workspace);
    }

    // ── Edge cases: from_config preserves tracker ────────────

    #[test]
    fn from_config_creates_fresh_tracker() {
        let autonomy_config = crate::config::AutonomyConfig {
            level: AutonomyLevel::Full,
            workspace_only: false,
            allowed_commands: vec![],
            forbidden_paths: vec![],
            max_actions_per_hour: 10,
            max_cost_per_day_cents: 100,
            require_approval_for_medium_risk: true,
            block_high_risk_commands: true,
            ..crate::config::AutonomyConfig::default()
        };
        let workspace = PathBuf::from("/tmp/test");
        let policy = SecurityPolicy::from_config(&autonomy_config, &workspace);
        assert_eq!(policy.tracker.count(), 0);
        assert!(!policy.is_rate_limited());
    }

    // ══════════════════════════════════════════════════════════
    // SECURITY CHECKLIST TESTS
    // Checklist: gateway not public, pairing required,
    //            filesystem scoped (no /), access via tunnel
    // ══════════════════════════════════════════════════════════

    // ── Checklist #3: Filesystem scoped (no /) ──────────────

    #[test]
    fn checklist_root_path_blocked() {
        let p = default_policy();
        assert!(!p.is_path_allowed("/"));
        assert!(!p.is_path_allowed("/anything"));
    }

    #[test]
    fn checklist_all_system_dirs_blocked() {
        let p = SecurityPolicy {
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        for dir in [
            "/etc", "/root", "/home", "/usr", "/bin", "/sbin", "/lib", "/opt", "/boot", "/dev",
            "/proc", "/sys", "/var",
        ] {
            assert!(
                !p.is_path_allowed(dir),
                "System dir should be blocked: {dir}"
            );
            assert!(
                !p.is_path_allowed(&format!("{dir}/subpath")),
                "Subpath of system dir should be blocked: {dir}/subpath"
            );
        }
    }

    #[test]
    fn checklist_sensitive_dotfiles_blocked() {
        let p = SecurityPolicy {
            workspace_only: false,
            ..SecurityPolicy::default()
        };
        for path in [
            "~/.ssh/id_rsa",
            "~/.gnupg/secring.gpg",
            "~/.aws/credentials",
            "~/.config/secrets",
        ] {
            assert!(
                !p.is_path_allowed(path),
                "Sensitive dotfile should be blocked: {path}"
            );
        }
    }

    #[test]
    fn checklist_null_byte_injection_blocked() {
        let p = default_policy();
        assert!(!p.is_path_allowed("safe\0/../../../etc/passwd"));
        assert!(!p.is_path_allowed("\0"));
        assert!(!p.is_path_allowed("file\0"));
    }

    #[test]
    fn checklist_workspace_only_blocks_all_absolute() {
        let p = SecurityPolicy {
            workspace_only: true,
            ..SecurityPolicy::default()
        };
        assert!(!p.is_path_allowed("/any/absolute/path"));
        assert!(p.is_path_allowed("relative/path.txt"));
    }

    #[test]
    fn checklist_resolved_path_must_be_in_workspace() {
        let p = SecurityPolicy {
            workspace_dir: PathBuf::from("/home/user/project"),
            ..SecurityPolicy::default()
        };
        // Inside workspace — allowed
        assert!(p.is_resolved_path_allowed(Path::new("/home/user/project/src/main.rs")));
        // Outside workspace — blocked (symlink escape)
        assert!(!p.is_resolved_path_allowed(Path::new("/etc/passwd")));
        assert!(!p.is_resolved_path_allowed(Path::new("/home/user/other_project/file")));
        // Root — blocked
        assert!(!p.is_resolved_path_allowed(Path::new("/")));
    }

    #[test]
    fn checklist_default_policy_is_workspace_only() {
        let p = SecurityPolicy::default();
        assert!(
            p.workspace_only,
            "Default policy must be workspace_only=true"
        );
    }

    #[test]
    fn checklist_default_forbidden_paths_comprehensive() {
        let p = SecurityPolicy::default();
        // Must contain all critical system dirs
        for dir in ["/etc", "/root", "/proc", "/sys", "/dev", "/var"] {
            assert!(
                p.forbidden_paths.iter().any(|f| f == dir),
                "Default forbidden_paths must include {dir}"
            );
        }
        // Must contain sensitive dotfiles
        for dot in ["~/.ssh", "~/.gnupg", "~/.aws"] {
            assert!(
                p.forbidden_paths.iter().any(|f| f == dot),
                "Default forbidden_paths must include {dot}"
            );
        }
    }

    // ── §1.2 Path resolution / symlink bypass tests ──────────

    #[test]
    fn resolved_path_blocks_outside_workspace() {
        let workspace = std::env::temp_dir().join("gloamy_test_resolved_path");
        let _ = std::fs::create_dir_all(&workspace);

        // Use the canonicalized workspace so starts_with checks match
        let canonical_workspace = workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.clone());

        let policy = SecurityPolicy {
            workspace_dir: canonical_workspace.clone(),
            ..SecurityPolicy::default()
        };

        // A resolved path inside the workspace should be allowed
        let inside = canonical_workspace.join("subdir").join("file.txt");
        assert!(
            policy.is_resolved_path_allowed(&inside),
            "path inside workspace should be allowed"
        );

        // A resolved path outside the workspace should be blocked
        let canonical_temp = std::env::temp_dir()
            .canonicalize()
            .unwrap_or_else(|_| std::env::temp_dir());
        let outside = canonical_temp.join("outside_workspace_gloamy");
        assert!(
            !policy.is_resolved_path_allowed(&outside),
            "path outside workspace must be blocked"
        );

        let _ = std::fs::remove_dir_all(&workspace);
    }

    #[test]
    fn resolved_path_blocks_root_escape() {
        let policy = SecurityPolicy {
            workspace_dir: PathBuf::from("/home/gloamy_user/project"),
            ..SecurityPolicy::default()
        };

        assert!(
            !policy.is_resolved_path_allowed(Path::new("/etc/passwd")),
            "resolved path to /etc/passwd must be blocked"
        );
        assert!(
            !policy.is_resolved_path_allowed(Path::new("/root/.bashrc")),
            "resolved path to /root/.bashrc must be blocked"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolved_path_blocks_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join("gloamy_test_symlink_escape");
        let workspace = root.join("workspace");
        let outside = root.join("outside_target");

        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        // Create a symlink inside workspace pointing outside
        let link_path = workspace.join("escape_link");
        symlink(&outside, &link_path).unwrap();

        let policy = SecurityPolicy {
            workspace_dir: workspace.clone(),
            ..SecurityPolicy::default()
        };

        // The resolved symlink target should be outside workspace
        let resolved = link_path.canonicalize().unwrap();
        assert!(
            !policy.is_resolved_path_allowed(&resolved),
            "symlink-resolved path outside workspace must be blocked"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn allowed_roots_permits_paths_outside_workspace() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join("gloamy_test_allowed_roots");
        let workspace = root.join("workspace");
        let extra = root.join("extra_root");
        let extra_file = extra.join("data.txt");

        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&extra).unwrap();
        std::fs::write(&extra_file, "test").unwrap();

        // Symlink inside workspace pointing to extra root
        let link_path = workspace.join("link_to_extra");
        symlink(&extra, &link_path).unwrap();

        let resolved = link_path.join("data.txt").canonicalize().unwrap();

        // Without allowed_roots — blocked (symlink escape)
        let policy_without = SecurityPolicy {
            workspace_dir: workspace.clone(),
            allowed_roots: vec![],
            ..SecurityPolicy::default()
        };
        assert!(
            !policy_without.is_resolved_path_allowed(&resolved),
            "without allowed_roots, symlink target must be blocked"
        );

        // With allowed_roots — permitted
        let policy_with = SecurityPolicy {
            workspace_dir: workspace.clone(),
            allowed_roots: vec![extra.clone()],
            ..SecurityPolicy::default()
        };
        assert!(
            policy_with.is_resolved_path_allowed(&resolved),
            "with allowed_roots containing the target, symlink must be allowed"
        );

        // Unrelated path still blocked
        let unrelated = root.join("unrelated");
        std::fs::create_dir_all(&unrelated).unwrap();
        assert!(
            !policy_with.is_resolved_path_allowed(&unrelated.canonicalize().unwrap()),
            "paths outside workspace and allowed_roots must still be blocked"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn is_path_allowed_blocks_null_bytes() {
        let policy = default_policy();
        assert!(
            !policy.is_path_allowed("file\0.txt"),
            "paths with null bytes must be blocked"
        );
    }

    #[test]
    fn is_path_allowed_blocks_url_encoded_traversal() {
        let policy = default_policy();
        assert!(
            !policy.is_path_allowed("..%2fetc%2fpasswd"),
            "URL-encoded path traversal must be blocked"
        );
        assert!(
            !policy.is_path_allowed("subdir%2f..%2f..%2fetc"),
            "URL-encoded parent dir traversal must be blocked"
        );
    }
}
