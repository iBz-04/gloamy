use super::gui_verify;
use super::traits::{
    GuiExpectation, GuiObservation, PreObservationStrategy, Tool, ToolResult, WaitStrategy,
};
use crate::approval::{ApprovalManager, ApprovalRequest, GuiApprovalContext};
use crate::config::GuiVerificationConfig;
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

const MAC_AUTOMATION_TIMEOUT_SECS: u64 = 20;
const MAX_APP_NAME_CHARS: usize = 120;
const MAX_SCRIPT_CHARS: usize = 8_000;
const MAX_OUTPUT_CHARS: usize = 8_000;
const DEFAULT_CLICK_SETTLE_MS: u64 = 150;
const MAX_MODIFIER_KEYS: usize = 4;
const MAX_FILTER_CHARS: usize = 200;
const DEFAULT_WINDOW_QUERY_SAMPLE_LIMIT: usize = 5;
const MAX_WINDOW_QUERY_RESULTS: usize = 50;
const FRONT_WINDOW_ROW_SEPARATOR: char = '\u{001e}';
const FRONT_WINDOW_FIELD_SEPARATOR: char = '\u{001f}';

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoordinateSpace {
    source_width: i64,
    source_height: i64,
    target_width: i64,
    target_height: i64,
    offset_x: i64,
    offset_y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedCoordinates {
    requested_x: i64,
    requested_y: i64,
    resolved_x: i64,
    resolved_y: i64,
    coordinate_space: Option<CoordinateSpace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontWindowElementQuery {
    role: Option<String>,
    title_contains: Option<String>,
    description_contains: Option<String>,
    value_contains: Option<String>,
}

/// macOS desktop automation helper (launch/activate apps and run AppleScript).
///
/// This tool provides an explicit, policy-aware path for local GUI automation
/// so the agent does not need to improvise via shell allowlist gaps.
pub struct MacAutomationTool {
    security: Arc<SecurityPolicy>,
    gui_verification: GuiVerificationConfig,
    gui_approval: Arc<ApprovalManager>,
}

impl MacAutomationTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self::new_with_gui_approval(
            security,
            GuiVerificationConfig::default(),
            Arc::new(ApprovalManager::from_config(
                &crate::config::AutonomyConfig::default(),
            )),
        )
    }

    pub fn new_with_gui_approval(
        security: Arc<SecurityPolicy>,
        gui_verification: GuiVerificationConfig,
        gui_approval: Arc<ApprovalManager>,
    ) -> Self {
        Self {
            security,
            gui_verification,
            gui_approval,
        }
    }

    fn parse_action(args: &serde_json::Value) -> anyhow::Result<&str> {
        args.get("action")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))
    }

    fn parse_app_name(args: &serde_json::Value) -> anyhow::Result<String> {
        let app_name = args
            .get("app_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'app_name' parameter"))?;

        if app_name.chars().count() > MAX_APP_NAME_CHARS {
            anyhow::bail!("app_name is too long (max {MAX_APP_NAME_CHARS} characters)");
        }
        if app_name.chars().any(char::is_control) {
            anyhow::bail!("app_name contains control characters");
        }

        Ok(app_name.to_string())
    }

    fn parse_applescript_lines(args: &serde_json::Value) -> anyhow::Result<Vec<String>> {
        if let Some(script) = args.get("script").and_then(serde_json::Value::as_str) {
            let trimmed = script.trim();
            if trimmed.is_empty() {
                anyhow::bail!("'script' cannot be empty");
            }
            if trimmed.chars().count() > MAX_SCRIPT_CHARS {
                anyhow::bail!("script is too long (max {MAX_SCRIPT_CHARS} characters)");
            }
            if trimmed.contains('\0') {
                anyhow::bail!("script contains a null byte");
            }
            return Ok(vec![trimmed.to_string()]);
        }

        let lines = args
            .get("script_lines")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("Provide either 'script' or 'script_lines'"))?;

        if lines.is_empty() {
            anyhow::bail!("'script_lines' cannot be empty");
        }

        let mut total_len = 0usize;
        let mut parsed = Vec::with_capacity(lines.len());
        for (idx, line) in lines.iter().enumerate() {
            let value = line
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("script_lines[{idx}] must be a non-empty string"))?;

            if value.contains('\0') {
                anyhow::bail!("script_lines[{idx}] contains a null byte");
            }

            total_len += value.len();
            if total_len > MAX_SCRIPT_CHARS {
                anyhow::bail!("script_lines total size exceeds {MAX_SCRIPT_CHARS} characters");
            }
            parsed.push(value.to_string());
        }

        Ok(parsed)
    }

    fn escape_applescript_literal(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ")
            .replace('\r', " ")
    }

    fn parse_coordinates(args: &serde_json::Value, action: &str) -> anyhow::Result<(i64, i64)> {
        let x = args
            .get("x")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("Missing 'x' coordinate for {action}"))?;
        let y = args
            .get("y")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("Missing 'y' coordinate for {action}"))?;
        Ok((x, y))
    }

    fn parse_optional_text_arg(
        args: &serde_json::Value,
        key: &str,
        max_chars: usize,
    ) -> anyhow::Result<Option<String>> {
        let Some(raw) = args.get(key) else {
            return Ok(None);
        };

        let value = raw
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'{key}' must be a string"))?
            .trim();
        if value.is_empty() {
            return Ok(None);
        }
        if value.chars().count() > max_chars {
            anyhow::bail!("'{key}' is too long (max {max_chars} characters)");
        }
        if value.chars().any(char::is_control) {
            anyhow::bail!("'{key}' contains control characters");
        }

        Ok(Some(value.to_string()))
    }

    fn parse_max_results_arg(args: &serde_json::Value) -> anyhow::Result<usize> {
        let Some(raw) = args.get("max_results") else {
            return Ok(DEFAULT_WINDOW_QUERY_SAMPLE_LIMIT);
        };

        let value = raw
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("'max_results' must be a positive integer"))?;
        if value == 0 {
            anyhow::bail!("'max_results' must be > 0");
        }
        if value > MAX_WINDOW_QUERY_RESULTS as u64 {
            anyhow::bail!("'max_results' exceeds {MAX_WINDOW_QUERY_RESULTS}");
        }

        Ok(value as usize)
    }

    fn parse_front_window_query(
        args: &serde_json::Value,
    ) -> anyhow::Result<FrontWindowElementQuery> {
        Ok(FrontWindowElementQuery {
            role: Self::parse_optional_text_arg(args, "role", MAX_FILTER_CHARS)?,
            title_contains: Self::parse_optional_text_arg(
                args,
                "title_contains",
                MAX_FILTER_CHARS,
            )?,
            description_contains: Self::parse_optional_text_arg(
                args,
                "description_contains",
                MAX_FILTER_CHARS,
            )?,
            value_contains: Self::parse_optional_text_arg(
                args,
                "value_contains",
                MAX_FILTER_CHARS,
            )?,
        })
    }

    fn front_window_query_from_key(key: &str) -> anyhow::Result<FrontWindowElementQuery> {
        let payload = key
            .strip_prefix("front_window_match_count::")
            .ok_or_else(|| anyhow::anyhow!("Invalid front window match key prefix"))?;
        let parsed: Value = serde_json::from_str(payload)
            .map_err(|error| anyhow::anyhow!("Invalid front window match key: {error}"))?;
        Ok(FrontWindowElementQuery {
            role: parsed
                .get("role")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            title_contains: parsed
                .get("title_contains")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            description_contains: parsed
                .get("description_contains")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            value_contains: parsed
                .get("value_contains")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
        })
    }

    fn front_window_query_json(query: &FrontWindowElementQuery) -> Value {
        json!({
            "role": query.role.as_deref(),
            "title_contains": query.title_contains.as_deref(),
            "description_contains": query.description_contains.as_deref(),
            "value_contains": query.value_contains.as_deref(),
        })
    }

    fn parse_modifier_keys(args: &serde_json::Value) -> anyhow::Result<Vec<String>> {
        let Some(raw) = args.get("keys") else {
            return Ok(Vec::new());
        };

        let values = raw
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'keys' must be an array of modifier names"))?;
        if values.len() > MAX_MODIFIER_KEYS {
            anyhow::bail!("'keys' accepts at most {MAX_MODIFIER_KEYS} modifiers");
        }

        let mut normalized = Vec::new();
        for (idx, value) in values.iter().enumerate() {
            let key = value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!("keys[{idx}] must be a non-empty string"))?
                .to_ascii_lowercase();
            let canonical = match key.as_str() {
                "option" | "alt" => "option",
                "shift" => "shift",
                "command" | "cmd" => "command",
                "control" | "ctrl" => "control",
                "function" | "fn" => "fn",
                other => {
                    anyhow::bail!(
                        "Unsupported modifier key '{other}'. Allowed: option, shift, command, control, fn"
                    )
                }
            };
            if !normalized.iter().any(|existing| existing == canonical) {
                normalized.push(canonical.to_string());
            }
        }

        Ok(normalized)
    }

    fn modifier_flag_expression(keys: &[String]) -> String {
        if keys.is_empty() {
            return "0".into();
        }

        keys.iter()
            .map(|key| match key.as_str() {
                "option" => "$.kCGEventFlagMaskAlternate",
                "shift" => "$.kCGEventFlagMaskShift",
                "command" => "$.kCGEventFlagMaskCommand",
                "control" => "$.kCGEventFlagMaskControl",
                "fn" => "$.kCGEventFlagMaskSecondaryFn",
                _ => "0",
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn scale_axis(value: i64, source_extent: i64, target_extent: i64) -> i64 {
        if source_extent <= 1 || target_extent <= 1 {
            0
        } else {
            let source_denominator = (source_extent - 1) as f64;
            let target_denominator = (target_extent - 1) as f64;
            ((value as f64 / source_denominator) * target_denominator).round() as i64
        }
    }

    async fn resolve_coordinates(
        &self,
        args: &serde_json::Value,
        action: &str,
    ) -> anyhow::Result<ResolvedCoordinates> {
        let (requested_x, requested_y) = Self::parse_coordinates(args, action)?;
        let Some(raw_space) = args.get("coordinate_space") else {
            return Ok(ResolvedCoordinates {
                requested_x,
                requested_y,
                resolved_x: requested_x,
                resolved_y: requested_y,
                coordinate_space: None,
            });
        };

        let map = raw_space
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("'coordinate_space' must be an object"))?;

        let desktop = Self::virtual_screen_bounds().await?;
        let desktop_x = desktop.get("x").and_then(Value::as_i64).unwrap_or_default();
        let desktop_y = desktop.get("y").and_then(Value::as_i64).unwrap_or_default();
        let desktop_width = desktop
            .get("width")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let desktop_height = desktop
            .get("height")
            .and_then(Value::as_i64)
            .unwrap_or_default();

        let source_width = map
            .get("source_width")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                anyhow::anyhow!("coordinate_space.source_width must be a positive integer")
            })?;
        let source_height = map
            .get("source_height")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                anyhow::anyhow!("coordinate_space.source_height must be a positive integer")
            })?;
        if source_width <= 0 || source_height <= 0 {
            anyhow::bail!("coordinate_space source dimensions must be > 0");
        }
        if requested_x < 0
            || requested_y < 0
            || requested_x >= source_width
            || requested_y >= source_height
        {
            anyhow::bail!(
                "Coordinates ({requested_x}, {requested_y}) are outside coordinate_space bounds {source_width}x{source_height}"
            );
        }

        let target_width = map
            .get("target_width")
            .and_then(Value::as_i64)
            .unwrap_or(desktop_width);
        let target_height = map
            .get("target_height")
            .and_then(Value::as_i64)
            .unwrap_or(desktop_height);
        if target_width <= 0 || target_height <= 0 {
            anyhow::bail!("coordinate_space target dimensions must be > 0");
        }

        let custom_target = map.contains_key("target_width") || map.contains_key("target_height");
        let offset_x = map
            .get("offset_x")
            .and_then(Value::as_i64)
            .unwrap_or(if custom_target { 0 } else { desktop_x });
        let offset_y = map
            .get("offset_y")
            .and_then(Value::as_i64)
            .unwrap_or(if custom_target { 0 } else { desktop_y });

        Ok(ResolvedCoordinates {
            requested_x,
            requested_y,
            resolved_x: offset_x + Self::scale_axis(requested_x, source_width, target_width),
            resolved_y: offset_y + Self::scale_axis(requested_y, source_height, target_height),
            coordinate_space: Some(CoordinateSpace {
                source_width,
                source_height,
                target_width,
                target_height,
                offset_x,
                offset_y,
            }),
        })
    }

    fn is_applescript_ui_lookup_failure(message: &str) -> bool {
        let lower = message.to_ascii_lowercase();
        let has_ax_target = lower.contains("button")
            || lower.contains("click")
            || lower.contains("ui element")
            || lower.contains("menu item")
            || lower.contains("checkbox")
            || lower.contains("pop up button")
            || lower.contains("scroll area");
        let has_ax_error = lower.contains("can't get")
            || lower.contains("cannot get")
            || lower.contains("doesn't understand")
            || lower.contains("invalid index")
            || lower.contains("not accessible")
            || lower.contains("axerror")
            || lower.contains("nselement");
        has_ax_target && has_ax_error
    }

    fn add_applescript_failure_hint(mut result: ToolResult) -> ToolResult {
        if result.success {
            return result;
        }

        let Some(reason) = result.error.as_deref() else {
            return result;
        };

        if !Self::is_applescript_ui_lookup_failure(reason) {
            return result;
        }

        let hint = "AppleScript could not find the target UI element. Many macOS apps use custom controls that are invisible to button and element queries. Take a screenshot, locate the control visually, then retry with mac_automation action=click_at using screen coordinates.";

        if !reason.contains(hint) {
            result.error = Some(format!("{reason}\nHint: {hint}"));
        }

        result
    }

    fn truncate_output(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let mut out = trimmed.to_string();
        if out.len() > MAX_OUTPUT_CHARS {
            out.truncate(out.floor_char_boundary(MAX_OUTPUT_CHARS));
            out.push_str("\n... [output truncated]");
        }
        out
    }

    async fn run_macos_command(program: &str, args: &[String]) -> anyhow::Result<ToolResult> {
        let result = tokio::time::timeout(
            Duration::from_secs(MAC_AUTOMATION_TIMEOUT_SECS),
            tokio::process::Command::new(program).args(args).output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = Self::truncate_output(&String::from_utf8_lossy(&output.stdout));
                let stderr = Self::truncate_output(&String::from_utf8_lossy(&output.stderr));
                if output.status.success() {
                    Ok(ToolResult {
                        success: true,
                        output: if stdout.is_empty() {
                            "ok".to_string()
                        } else {
                            stdout
                        },
                        error: None,
                    })
                } else {
                    Ok(ToolResult {
                        success: false,
                        output: stdout,
                        error: Some(if stderr.is_empty() {
                            format!("{program} exited with status {}", output.status)
                        } else {
                            stderr
                        }),
                    })
                }
            }
            Ok(Err(error)) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute {program}: {error}")),
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "{program} timed out after {MAC_AUTOMATION_TIMEOUT_SECS}s"
                )),
            }),
        }
    }

    async fn virtual_screen_bounds() -> anyhow::Result<Value> {
        let script = r#"
ObjC.import('AppKit');
var screens = $.NSScreen.screens;
var count = Number(screens.count);
if (!count || count <= 0) {
    JSON.stringify({"x": 0, "y": 0, "width": 0, "height": 0});
} else {
    var minX = Infinity;
    var minY = Infinity;
    var maxX = -Infinity;
    var maxY = -Infinity;
    for (var i = 0; i < count; i++) {
        var frame = screens.objectAtIndex(i).frame;
        var x = Math.round(Number(frame.origin.x));
        var y = Math.round(Number(frame.origin.y));
        var width = Math.round(Number(frame.size.width));
        var height = Math.round(Number(frame.size.height));
        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        maxX = Math.max(maxX, x + width);
        maxY = Math.max(maxY, y + height);
    }
    JSON.stringify({
        "x": minX,
        "y": minY,
        "width": Math.max(0, maxX - minX),
        "height": Math.max(0, maxY - minY)
    });
}
"#;
        let result = Self::run_macos_command(
            "osascript",
            &[
                "-l".to_string(),
                "JavaScript".to_string(),
                "-e".to_string(),
                script.trim().to_string(),
            ],
        )
        .await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }

        serde_json::from_str(result.output.trim())
            .map_err(|error| anyhow::anyhow!("Failed to parse virtual screen bounds: {error}"))
    }

    fn resolved_coordinates_json(coords: &ResolvedCoordinates) -> Value {
        json!({
            "requested": {
                "x": coords.requested_x,
                "y": coords.requested_y,
            },
            "resolved": {
                "x": coords.resolved_x,
                "y": coords.resolved_y,
            },
            "coordinate_space": coords.coordinate_space.as_ref().map(|space| {
                json!({
                    "source_width": space.source_width,
                    "source_height": space.source_height,
                    "target_width": space.target_width,
                    "target_height": space.target_height,
                    "offset_x": space.offset_x,
                    "offset_y": space.offset_y,
                })
            }),
        })
    }

    fn front_window_query_script(query: &FrontWindowElementQuery, sample_limit: usize) -> String {
        let role = Self::escape_applescript_literal(query.role.as_deref().unwrap_or_default());
        let title =
            Self::escape_applescript_literal(query.title_contains.as_deref().unwrap_or_default());
        let description = Self::escape_applescript_literal(
            query.description_contains.as_deref().unwrap_or_default(),
        );
        let value =
            Self::escape_applescript_literal(query.value_contains.as_deref().unwrap_or_default());
        let sample_limit = sample_limit.max(1);

        format!(
            r#"
set fieldSep to character id {field_sep}
set rowSep to character id {row_sep}
set roleFilter to "{role}"
set titleFilter to "{title}"
set descriptionFilter to "{description}"
set valueFilter to "{value}"
set sampleLimit to {sample_limit}

on replace_text(find_text, replace_text, source_text)
    set AppleScript's text item delimiters to find_text
    set text_items to every text item of source_text
    set AppleScript's text item delimiters to replace_text
    set replaced_text to text_items as text
    set AppleScript's text item delimiters to ""
    return replaced_text
end replace_text

on sanitize_text(raw_text, fieldSep, rowSep)
    set text_value to raw_text as text
    set text_value to my replace_text(return, " ", text_value)
    set text_value to my replace_text(linefeed, " ", text_value)
    set text_value to my replace_text(fieldSep, " ", text_value)
    set text_value to my replace_text(rowSep, " ", text_value)
    return text_value
end sanitize_text

on matches_filter(actual_value, filter_value)
    if filter_value is "" then return true
    ignoring case
        return actual_value contains filter_value
    end ignoring
end matches_filter

tell application "System Events"
    tell (first process whose frontmost is true)
        set totalMatches to 0
        set sampleRows to {{}}
        try
            set uiItems to entire contents of front window
        on error
            set uiItems to {{}}
        end try

        repeat with uiElem in uiItems
            set roleText to ""
            set titleText to ""
            set descriptionText to ""
            set valueText to ""
            try
                set roleText to my sanitize_text((role of uiElem) as text, fieldSep, rowSep)
            end try
            try
                set titleText to my sanitize_text((title of uiElem) as text, fieldSep, rowSep)
            end try
            try
                set descriptionText to my sanitize_text((description of uiElem) as text, fieldSep, rowSep)
            end try
            try
                set valueText to my sanitize_text((value of uiElem) as text, fieldSep, rowSep)
            end try

            if my matches_filter(roleText, roleFilter) and my matches_filter(titleText, titleFilter) and my matches_filter(descriptionText, descriptionFilter) and my matches_filter(valueText, valueFilter) then
                set totalMatches to totalMatches + 1
                if (count of sampleRows) < sampleLimit then
                    set end of sampleRows to (roleText & fieldSep & titleText & fieldSep & descriptionText & fieldSep & valueText)
                end if
            end if
        end repeat

        set outputRows to {{totalMatches as text}}
        repeat with rowText in sampleRows
            set end of outputRows to contents of rowText
        end repeat
        set AppleScript's text item delimiters to rowSep
        set joinedOutput to outputRows as text
        set AppleScript's text item delimiters to ""
        return joinedOutput
    end tell
end tell
"#,
            field_sep = FRONT_WINDOW_FIELD_SEPARATOR as u32,
            row_sep = FRONT_WINDOW_ROW_SEPARATOR as u32,
            role = role,
            title = title,
            description = description,
            value = value,
            sample_limit = sample_limit,
        )
    }

    async fn front_window_element_match_report(
        &self,
        query: &FrontWindowElementQuery,
        sample_limit: usize,
    ) -> anyhow::Result<Value> {
        let script = Self::front_window_query_script(query, sample_limit);
        let result = Self::run_macos_command("osascript", &["-e".to_string(), script]).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }

        let mut rows = result.output.split(FRONT_WINDOW_ROW_SEPARATOR);
        let count = rows
            .next()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or_default();
        let samples = rows
            .filter(|row| !row.trim().is_empty())
            .map(|row| {
                let mut fields = row.split(FRONT_WINDOW_FIELD_SEPARATOR);
                json!({
                    "role": fields.next().unwrap_or_default(),
                    "title": fields.next().unwrap_or_default(),
                    "description": fields.next().unwrap_or_default(),
                    "value": fields.next().unwrap_or_default(),
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "query": Self::front_window_query_json(query),
            "count": count,
            "samples": samples,
        }))
    }

    async fn apply_unverified_wait(&self, wait_strategy: &WaitStrategy) -> anyhow::Result<()> {
        match wait_strategy {
            WaitStrategy::None => {
                tokio::time::sleep(Duration::from_millis(DEFAULT_CLICK_SETTLE_MS)).await;
                Ok(())
            }
            WaitStrategy::FixedMs { ms } => {
                tokio::time::sleep(Duration::from_millis(*ms)).await;
                Ok(())
            }
            WaitStrategy::AccessibilityEvent {
                notification,
                timeout_ms,
            } => {
                self.apply_accessibility_event_wait(notification, *timeout_ms)
                    .await
            }
            WaitStrategy::PollUntilVerified { .. } => {
                anyhow::bail!(
                    "wait.strategy=poll_until_verified requires 'expect' on mac_automation"
                )
            }
            WaitStrategy::DomEvent { .. } | WaitStrategy::SelectorPresent { .. } => {
                anyhow::bail!("Unsupported wait strategy for mac_automation")
            }
        }
    }

    async fn collect_gui_evidence(&self, keys: &[String]) -> anyhow::Result<Value> {
        let mut evidence = json!({});

        if keys.iter().any(|key| key == "title") {
            let title = self.frontmost_window_title().await.unwrap_or_default();
            merge_json_objects(&mut evidence, &json!({ "title": title }));
        }

        if keys.iter().any(|key| key == "dialog_present") {
            let present = self.frontmost_dialog_present().await.unwrap_or(false);
            merge_json_objects(&mut evidence, &json!({ "dialog_present": present }));
        }

        if keys.iter().any(|key| key == "focused_element") {
            if let Ok(info) = self.focused_element_info().await {
                merge_json_objects(&mut evidence, &json!({ "focused_element": info }));
            }
        }

        // Handle hit_test_result for coordinate-based element probing
        if keys.iter().any(|key| key == "hit_test_result") {
            // hit_test coordinates must be passed via evidence context; skip if not available
        }

        // Handle ax_attributes.* keys
        let ax_keys: Vec<&str> = keys
            .iter()
            .filter_map(|k| k.strip_prefix("ax_attributes."))
            .collect();
        if !ax_keys.is_empty() {
            let mut attrs = json!({});
            for attr_name in ax_keys {
                if let Ok(val) = self.ax_attribute_of_focused(attr_name).await {
                    if let Value::Object(ref mut map) = attrs {
                        map.insert(attr_name.to_string(), json!(val));
                    }
                }
            }
            merge_json_objects(&mut evidence, &json!({ "ax_attributes": attrs }));
        }

        let window_match_keys: Vec<&String> = keys
            .iter()
            .filter(|key| key.starts_with("front_window_match_count::"))
            .collect();
        if !window_match_keys.is_empty() {
            let mut counts = json!({});
            for key in window_match_keys {
                let query = match Self::front_window_query_from_key(key) {
                    Ok(query) => query,
                    Err(_) => continue,
                };
                if let Ok(report) = self
                    .front_window_element_match_report(&query, DEFAULT_WINDOW_QUERY_SAMPLE_LIMIT)
                    .await
                {
                    if let Value::Object(ref mut map) = counts {
                        map.insert(key.clone(), report);
                    }
                }
            }
            merge_json_objects(
                &mut evidence,
                &json!({ "front_window_match_counts": counts }),
            );
        }

        Ok(evidence)
    }

    async fn maybe_collect_pre_observation(
        &self,
        strategy: &PreObservationStrategy,
        expectations: &[GuiExpectation],
    ) -> Option<GuiObservation> {
        let keys = match strategy {
            PreObservationStrategy::None => {
                if gui_verify::expectations_require_pre_observation(expectations) {
                    gui_verify::infer_evidence_keys(expectations)
                } else {
                    Vec::new()
                }
            }
            PreObservationStrategy::Auto => gui_verify::infer_evidence_keys(expectations),
            PreObservationStrategy::Explicit { keys } => keys.clone(),
        };

        if keys.is_empty() {
            return None;
        }

        match self.collect_gui_evidence(&keys).await {
            Ok(evidence) => Some(gui_verify::observation("mac_pre", evidence)),
            Err(_) => None,
        }
    }

    fn validate_wait_strategy(&self, wait_strategy: &WaitStrategy) -> anyhow::Result<()> {
        match wait_strategy {
            WaitStrategy::None
            | WaitStrategy::FixedMs { .. }
            | WaitStrategy::PollUntilVerified { .. }
            | WaitStrategy::AccessibilityEvent { .. } => Ok(()),
            WaitStrategy::DomEvent { .. } => anyhow::bail!(
                "wait.strategy=dom_event is browser-specific and unsupported by mac_automation"
            ),
            WaitStrategy::SelectorPresent { .. } => anyhow::bail!(
                "wait.strategy=selector_present is browser-specific and unsupported by mac_automation"
            ),
        }
    }

    /// Wait for a macOS accessibility state change by polling via osascript.
    ///
    /// Supported notifications (mapped to observable checks):
    /// - `AXTitleChanged` — polls frontmost window title until it differs from baseline.
    /// - `AXFocusedUIElementChanged` — polls frontmost app name until it differs.
    /// - `AXSheetCreated` / `AXWindowCreated` — polls for dialog/sheet presence.
    /// - `AXValueChanged` — generic; polls title as a proxy.
    ///
    /// Falls back to a fixed sleep for unrecognized notification names.
    async fn apply_accessibility_event_wait(
        &self,
        notification: &str,
        timeout_ms: u64,
    ) -> anyhow::Result<()> {
        const POLL_INTERVAL_MS: u64 = 200;

        let baseline = match notification {
            "AXTitleChanged" | "AXValueChanged" => self.frontmost_window_title().await.ok(),
            "AXFocusedUIElementChanged" => self.frontmost_app_name().await.ok(),
            "AXSheetCreated" | "AXWindowCreated" => {
                let present = self.frontmost_dialog_present().await.unwrap_or(false);
                Some(present.to_string())
            }
            _ => {
                // Unrecognized notification: fall back to a bounded sleep.
                let capped = timeout_ms.min(10_000);
                tokio::time::sleep(Duration::from_millis(capped)).await;
                return Ok(());
            }
        };

        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;

            let current = match notification {
                "AXTitleChanged" | "AXValueChanged" => self.frontmost_window_title().await.ok(),
                "AXFocusedUIElementChanged" => self.frontmost_app_name().await.ok(),
                "AXSheetCreated" | "AXWindowCreated" => {
                    let present = self.frontmost_dialog_present().await.unwrap_or(false);
                    Some(present.to_string())
                }
                _ => None,
            };

            if current != baseline {
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                anyhow::bail!(
                    "accessibility_event wait for '{notification}' timed out after {timeout_ms}ms"
                );
            }
        }
    }

    /// Get the name of the frontmost application.
    async fn frontmost_app_name(&self) -> anyhow::Result<String> {
        let args = vec![
            "-e".to_string(),
            "tell application \"System Events\"".to_string(),
            "-e".to_string(),
            "return name of first process whose frontmost is true".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
        ];
        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output)
    }

    async fn maybe_request_gui_approval(
        &self,
        action: &str,
        args: &Value,
        expectations: &[GuiExpectation],
        pre_observation: Option<&GuiObservation>,
    ) -> anyhow::Result<Option<ToolResult>> {
        let reversibility =
            gui_verify::classify_reversibility(self.name(), action, args, expectations);
        if !gui_verify::needs_gui_approval(
            &self.gui_verification,
            self.security.autonomy,
            reversibility,
        ) {
            return Ok(None);
        }

        let action_summary = summarize_mac_action(action, args);

        if self
            .gui_approval
            .has_gui_preapproval(self.name(), &action_summary)
        {
            return Ok(None);
        }

        let current_state = if let Some(observation) = pre_observation {
            Some(observation.evidence.clone())
        } else {
            match self
                .collect_gui_evidence(&default_mac_approval_keys())
                .await
            {
                Ok(evidence) if !evidence.is_null() => Some(evidence),
                Ok(_) => None,
                Err(_) => None,
            }
        };

        let request = ApprovalRequest {
            tool_name: self.name().to_string(),
            arguments: args.clone(),
            gui_context: Some(GuiApprovalContext {
                action_summary,
                reversibility,
                current_state,
                expected_outcome: expectations
                    .iter()
                    .map(gui_verify::describe_expectation)
                    .collect(),
                screenshot_path: None,
            }),
        };

        let decision = self
            .gui_approval
            .request_gui_approval(&request, self.gui_verification.approval_timeout_secs);

        if decision == crate::approval::ApprovalResponse::No {
            return Ok(Some(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: GUI approval denied".into()),
            }));
        }

        Ok(None)
    }

    async fn frontmost_window_title(&self) -> anyhow::Result<String> {
        let args = vec![
            "-e".to_string(),
            "tell application \"System Events\"".to_string(),
            "-e".to_string(),
            "tell (first process whose frontmost is true)".to_string(),
            "-e".to_string(),
            "try".to_string(),
            "-e".to_string(),
            "return name of front window".to_string(),
            "-e".to_string(),
            "on error".to_string(),
            "-e".to_string(),
            "return \"\"".to_string(),
            "-e".to_string(),
            "end try".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
        ];

        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output)
    }

    /// Query the focused UI element's role, title, and description via AX.
    async fn focused_element_info(&self) -> anyhow::Result<serde_json::Value> {
        let script = r#"
tell application "System Events"
    tell (first process whose frontmost is true)
        try
            set fe to focused UI element
            set r to role of fe
            set t to ""
            set d to ""
            try
                set t to title of fe
            end try
            try
                set d to description of fe
            end try
            try
                set v to value of fe
            on error
                set v to ""
            end try
            return r & "|||" & t & "|||" & d & "|||" & v
        on error
            return "unknown|||||||"
        end try
    end tell
end tell
"#;
        let args = vec!["-e".to_string(), script.trim().to_string()];
        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        let parts: Vec<&str> = result.output.trim().splitn(4, "|||").collect();
        Ok(json!({
            "role": parts.first().unwrap_or(&""),
            "title": parts.get(1).unwrap_or(&""),
            "description": parts.get(2).unwrap_or(&""),
            "value": parts.get(3).unwrap_or(&""),
        }))
    }

    /// Query the AX element at a screen coordinate via System Events.
    async fn ax_element_at_point(&self, x: i64, y: i64) -> anyhow::Result<serde_json::Value> {
        // Use AppleScript + System Events to probe the element at a coordinate.
        // macOS accessibility does not expose a direct "element at point" via osascript,
        // so we use the focused-app's UI element tree with position/size matching.
        // For best fidelity, we use `cliclick` to move the mouse, then read focused element.
        // However, as a non-destructive probe, we use a JXA script that walks the AX tree.
        let script = format!(
            r#"
ObjC.import('ApplicationServices');
var pt = $.CGPointMake({x}, {y});
var el = $.AXUIElementCreateSystemWide();
var ref = Ref();
var err = $.AXUIElementCopyElementAtPosition(el, pt.x, pt.y, ref);
if (err !== 0) {{
    JSON.stringify({{"error": "AXUIElementCopyElementAtPosition failed", "code": err}});
}} else {{
    var elem = ref[0];
    function getAttr(e, attr) {{
        var v = Ref();
        var r = $.AXUIElementCopyAttributeValue(e, attr, v);
        if (r === 0 && v[0] != null) return String(v[0]);
        return "";
    }}
    JSON.stringify({{
        "role": getAttr(elem, "AXRole"),
        "label": getAttr(elem, "AXTitle"),
        "description": getAttr(elem, "AXDescription"),
        "value": getAttr(elem, "AXValue"),
    }});
}}
"#
        );
        let args = vec![
            "-l".to_string(),
            "JavaScript".to_string(),
            "-e".to_string(),
            script.trim().to_string(),
        ];
        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        let parsed: serde_json::Value = serde_json::from_str(result.output.trim())
            .unwrap_or_else(|_| json!({"raw_output": result.output.trim()}));
        Ok(parsed)
    }

    /// Read a specific AX attribute of the frontmost focused element.
    async fn ax_attribute_of_focused(&self, attribute: &str) -> anyhow::Result<String> {
        let escaped = Self::escape_applescript_literal(attribute);
        let script = format!(
            r#"
tell application "System Events"
    tell (first process whose frontmost is true)
        try
            set fe to focused UI element
            return value of attribute "{escaped}" of fe
        on error
            return ""
        end try
    end tell
end tell
"#
        );
        let args = vec!["-e".to_string(), script.trim().to_string()];
        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output.trim().to_string())
    }

    /// Click at screen coordinates using osascript + CoreGraphics events.
    /// This is a real pixel-level click, not dependent on AX button lookup.
    async fn click_at_coordinate(
        x: i64,
        y: i64,
        button: &str,
        modifier_keys: &[String],
    ) -> anyhow::Result<ToolResult> {
        let (mouse_down, mouse_up, mouse_button, label) = match button {
            "right" => (
                "$.kCGEventRightMouseDown",
                "$.kCGEventRightMouseUp",
                "$.kCGMouseButtonRight",
                "right-clicked",
            ),
            _ => (
                "$.kCGEventLeftMouseDown",
                "$.kCGEventLeftMouseUp",
                "$.kCGMouseButtonLeft",
                "clicked",
            ),
        };
        let flags = Self::modifier_flag_expression(modifier_keys);
        let script = format!(
            r#"
ObjC.import('CoreGraphics');
var point = $.CGPointMake({x}, {y});
var flags = {flags};
$.CGWarpMouseCursorPosition(point);
var mouseDown = $.CGEventCreateMouseEvent(null, {mouse_down}, point, {mouse_button});
var mouseUp = $.CGEventCreateMouseEvent(null, {mouse_up}, point, {mouse_button});
if (flags !== 0) {{
    $.CGEventSetFlags(mouseDown, flags);
    $.CGEventSetFlags(mouseUp, flags);
}}
$.CGEventPost($.kCGHIDEventTap, mouseDown);
delay(0.05);
$.CGEventPost($.kCGHIDEventTap, mouseUp);
"{label}"
"#,
            flags = flags,
            mouse_down = mouse_down,
            mouse_up = mouse_up,
            mouse_button = mouse_button,
            label = label
        );
        let args = vec![
            "-l".to_string(),
            "JavaScript".to_string(),
            "-e".to_string(),
            script.trim().to_string(),
        ];
        Self::run_macos_command("osascript", &args).await
    }

    /// Move the real mouse cursor to screen coordinates and emit a hover event.
    async fn move_mouse_to_coordinate(x: i64, y: i64) -> anyhow::Result<ToolResult> {
        let script = format!(
            r#"
ObjC.import('CoreGraphics');
var point = $.CGPointMake({x}, {y});
$.CGWarpMouseCursorPosition(point);
var move = $.CGEventCreateMouseEvent(null, $.kCGEventMouseMoved, point, $.kCGMouseButtonLeft);
$.CGEventPost($.kCGHIDEventTap, move);
"moved"
"#
        );
        let args = vec![
            "-l".to_string(),
            "JavaScript".to_string(),
            "-e".to_string(),
            script.trim().to_string(),
        ];
        Self::run_macos_command("osascript", &args).await
    }

    /// Determine if a run_applescript invocation contains mutating UI operations.
    fn is_mutating_applescript(args: &serde_json::Value) -> bool {
        let script = args
            .get("script")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                args.get("script_lines")
                    .and_then(serde_json::Value::as_array)
                    .map(|lines| {
                        lines
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
            })
            .unwrap_or_default()
            .to_ascii_lowercase();

        if script.is_empty() {
            return false;
        }

        // Patterns that indicate the script modifies UI state
        [
            "click",
            "keystroke",
            "key code",
            "perform action",
            "set value",
            "set focused",
            "press",
            "pick",
            "select",
            "drag",
            "increment",
            "decrement",
            "confirm",
            "dismiss",
        ]
        .iter()
        .any(|needle| script.contains(needle))
    }

    async fn frontmost_dialog_present(&self) -> anyhow::Result<bool> {
        let args = vec![
            "-e".to_string(),
            "tell application \"System Events\"".to_string(),
            "-e".to_string(),
            "tell (first process whose frontmost is true)".to_string(),
            "-e".to_string(),
            "try".to_string(),
            "-e".to_string(),
            "if exists (sheet 1 of front window) then return \"true\"".to_string(),
            "-e".to_string(),
            "end try".to_string(),
            "-e".to_string(),
            "return \"false\"".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
        ];

        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output.trim().eq_ignore_ascii_case("true"))
    }
}

#[async_trait]
impl Tool for MacAutomationTool {
    fn name(&self) -> &str {
        "mac_automation"
    }

    fn description(&self) -> &str {
        "macOS desktop automation: launch/activate apps, run AppleScript, move the pointer, click at screen coordinates, and inspect accessibility state. Use this tool (not screenshot) when the user asks to interact with an app, for example to hover a custom control, click a shutter/capture button in Photo Booth, press record in QuickTime, or inspect front-window elements before verifying a native app action. click_at supports modifier keys and coordinate-space scaling for screenshot-derived points. For mutating actions (move_mouse, click_at, run_applescript with click/keystroke), provide 'expect' to verify the action succeeded. Without 'expect', mutating actions return ambiguous verification status instead of claiming proof."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["launch_app", "activate_app", "run_applescript", "move_mouse", "click_at", "read_focused_element", "hit_test", "inspect_window_elements"],
                    "description": "Automation action to execute. move_mouse: move the real mouse pointer to screen coordinates and emit a hover event. click_at: pixel-level click at screen coordinates with optional modifier keys and coordinate-space scaling. read_focused_element: query the focused AX element's role/title/description/value. hit_test: probe the AX element at a screen coordinate without clicking. inspect_window_elements: inspect and filter front-window accessibility elements for verification planning."
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name for launch_app/activate_app (e.g., 'MongoDB Compass')"
                },
                "script": {
                    "type": "string",
                    "description": "AppleScript source for run_applescript"
                },
                "script_lines": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "AppleScript passed as multiple lines (-e per line)"
                },
                "x": {
                    "type": "integer",
                    "description": "Screen X coordinate for move_mouse, click_at, and hit_test actions"
                },
                "y": {
                    "type": "integer",
                    "description": "Screen Y coordinate for move_mouse, click_at, and hit_test actions"
                },
                "button": {
                    "type": "string",
                    "enum": ["left", "right"],
                    "description": "Mouse button for click_at (default: left)"
                },
                "keys": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["option", "alt", "shift", "command", "cmd", "control", "ctrl", "fn", "function"]
                    },
                    "description": "Optional modifier keys to hold during click_at, such as [\"option\"] for Photo Booth capture without countdown or [\"option\", \"shift\"] to disable countdown and flash."
                },
                "coordinate_space": {
                    "type": "object",
                    "description": "Optional coordinate normalization metadata for screenshot-derived coordinates. x/y are interpreted within source_width/source_height, then scaled into the current desktop or the specified target rectangle.",
                    "properties": {
                        "source_width": { "type": "integer", "minimum": 1 },
                        "source_height": { "type": "integer", "minimum": 1 },
                        "target_width": { "type": "integer", "minimum": 1 },
                        "target_height": { "type": "integer", "minimum": 1 },
                        "offset_x": { "type": "integer" },
                        "offset_y": { "type": "integer" }
                    },
                    "required": ["source_width", "source_height"]
                },
                "role": {
                    "type": "string",
                    "description": "Optional front-window accessibility role filter for inspect_window_elements or front_window_element_count_changed verification."
                },
                "title_contains": {
                    "type": "string",
                    "description": "Optional front-window title substring filter for inspect_window_elements or front_window_element_count_changed verification."
                },
                "description_contains": {
                    "type": "string",
                    "description": "Optional front-window description substring filter for inspect_window_elements or front_window_element_count_changed verification."
                },
                "value_contains": {
                    "type": "string",
                    "description": "Optional front-window value substring filter for inspect_window_elements or front_window_element_count_changed verification."
                },
                "max_results": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_WINDOW_QUERY_RESULTS,
                    "description": "Maximum number of sample elements returned by inspect_window_elements."
                },
                "expect": {
                    "description": "Verification expectation(s) for mutating actions. When provided, success means the expected UI state was verified, not just that the command exited 0. Accepts a single object or array. Supported kinds: field_value_equals, focused_element_is, checkbox_checked, window_title_contains, dialog_present, url_is, url_host_is, file_exists, download_completed, front_window_element_count_changed, element_at_coordinate, ax_attribute_equals.",
                    "oneOf": [
                        { "type": "object", "properties": { "kind": { "type": "string" } }, "required": ["kind"] },
                        { "type": "array", "items": { "type": "object", "properties": { "kind": { "type": "string" } }, "required": ["kind"] } }
                    ]
                },
                "pre_observe": {
                    "description": "Optional pre-action observation strategy. Use \"auto\" to infer evidence keys from expectations or provide {\"keys\": [...]}."
                },
                "wait": {
                    "description": "Optional wait strategy object. Example: {\"strategy\":\"fixed_ms\",\"ms\":500}."
                },
                "reversibility": {
                    "type": "string",
                    "enum": ["reversible", "partially_reversible", "irreversible", "unknown"],
                    "description": "Optional caller override for GUI action reversibility classification."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // Parse action first so we can gate read vs mutating operations correctly.
        let action = match Self::parse_action(&args) {
            Ok(action) => action,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                })
            }
        };

        let is_read_action = matches!(
            action,
            "read_focused_element" | "hit_test" | "inspect_window_elements"
        );

        // Read-only actions bypass autonomy and rate-limit gates (no side effects).
        if !is_read_action {
            if !self.security.can_act() {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Action blocked: autonomy is read-only".into()),
                });
            }

            if !self.security.record_action() {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Action blocked: rate limit exceeded".into()),
                });
            }
        }

        // Parse verification expectations (if any) before executing
        let expectations = match gui_verify::parse_expectations(&args) {
            Ok(exps) => exps,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                });
            }
        };

        let pre_observe = match gui_verify::parse_pre_observation_strategy(&args) {
            Ok(strategy) => strategy,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                });
            }
        };

        let wait_strategy = match gui_verify::parse_wait_strategy(&args) {
            Ok(strategy) => strategy,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                });
            }
        };

        if let Err(error) = self.validate_wait_strategy(&wait_strategy) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(error.to_string()),
            });
        }

        let expectations = match expectations {
            Some(exps) => exps,
            None => Vec::new(),
        };
        let pre_observation = self
            .maybe_collect_pre_observation(&pre_observe, &expectations)
            .await;

        let raw_result = match action {
            "launch_app" => {
                let app_name = match Self::parse_app_name(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                if let Some(result) = self
                    .maybe_request_gui_approval(
                        action,
                        &args,
                        &expectations,
                        pre_observation.as_ref(),
                    )
                    .await?
                {
                    return Ok(result);
                }
                let launch = Self::run_macos_command(
                    "open",
                    &["-a".to_string(), app_name.clone()],
                )
                .await?;

                ToolResult {
                    success: launch.success,
                    output: if launch.success {
                        format!("Launched app: {app_name}")
                    } else {
                        launch.output
                    },
                    error: launch.error,
                }
            }
            "activate_app" => {
                let app_name = match Self::parse_app_name(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                if let Some(result) = self
                    .maybe_request_gui_approval(
                        action,
                        &args,
                        &expectations,
                        pre_observation.as_ref(),
                    )
                    .await?
                {
                    return Ok(result);
                }
                let script = format!(
                    "tell application \"{}\" to activate",
                    Self::escape_applescript_literal(&app_name)
                );
                let activation = Self::run_macos_command(
                    "osascript",
                    &["-e".to_string(), script],
                )
                .await?;

                ToolResult {
                    success: activation.success,
                    output: if activation.success {
                        format!("Activated app: {app_name}")
                    } else {
                        activation.output
                    },
                    error: activation.error,
                }
            }
            "run_applescript" => {
                let script_lines = match Self::parse_applescript_lines(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                if let Some(result) = self
                    .maybe_request_gui_approval(
                        action,
                        &args,
                        &expectations,
                        pre_observation.as_ref(),
                    )
                    .await?
                {
                    return Ok(result);
                }
                let script_args: Vec<String> = script_lines
                    .into_iter()
                    .flat_map(|line| ["-e".to_string(), line])
                    .collect();

                let result = Self::run_macos_command("osascript", &script_args).await?;
                Self::add_applescript_failure_hint(result)
            }
            "move_mouse" => {
                let coords = match self.resolve_coordinates(&args, "move_mouse").await {
                    Ok(coords) => coords,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                if let Some(result) = self
                    .maybe_request_gui_approval(
                        action,
                        &args,
                        &expectations,
                        pre_observation.as_ref(),
                    )
                    .await?
                {
                    return Ok(result);
                }

                let move_result =
                    Self::move_mouse_to_coordinate(coords.resolved_x, coords.resolved_y).await?;
                if move_result.success {
                    if expectations.is_empty() {
                        if let Err(error) = self.apply_unverified_wait(&wait_strategy).await {
                            return Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(error.to_string()),
                            });
                        }
                    }

                    let hover_target = self
                        .ax_element_at_point(coords.resolved_x, coords.resolved_y)
                        .await
                        .unwrap_or(json!({}));
                    ToolResult {
                        success: true,
                        output: serde_json::to_string_pretty(&json!({
                            "moved": Self::resolved_coordinates_json(&coords),
                            "hover_target": hover_target,
                        }))
                        .unwrap_or_default(),
                        error: None,
                    }
                } else {
                    move_result
                }
            }
            "click_at" => {
                let coords = match self.resolve_coordinates(&args, "click_at").await {
                    Ok(coords) => coords,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                let modifier_keys = match Self::parse_modifier_keys(&args) {
                    Ok(keys) => keys,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                if let Some(result) = self
                    .maybe_request_gui_approval(
                        action,
                        &args,
                        &expectations,
                        pre_observation.as_ref(),
                    )
                    .await?
                {
                    return Ok(result);
                }

                let button = args
                    .get("button")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("left");
                let click_result = Self::click_at_coordinate(
                    coords.resolved_x,
                    coords.resolved_y,
                    button,
                    &modifier_keys,
                )
                .await?;

                if click_result.success {
                    if expectations.is_empty() {
                        if let Err(error) = self.apply_unverified_wait(&wait_strategy).await {
                            return Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(error.to_string()),
                            });
                        }
                    }

                    // Auto-collect focused element as evidence of what the click hit
                    let focused = self.focused_element_info().await.unwrap_or(json!({}));
                    ToolResult {
                        success: true,
                        output: serde_json::to_string_pretty(&json!({
                            "clicked": Self::resolved_coordinates_json(&coords),
                            "button": button,
                            "modifier_keys": modifier_keys,
                            "focused_after_click": focused,
                        }))
                        .unwrap_or_default(),
                        error: None,
                    }
                } else {
                    click_result
                }
            }
            "read_focused_element" => {
                // Pure read action — no approval needed, no mutation
                match self.focused_element_info().await {
                    Ok(info) => ToolResult {
                        success: true,
                        output: serde_json::to_string_pretty(&info).unwrap_or_default(),
                        error: None,
                    },
                    Err(e) => ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to read focused element: {e}")),
                    },
                }
            }
            "hit_test" => {
                let coords = match self.resolve_coordinates(&args, "hit_test").await {
                    Ok(coords) => coords,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                // Pure read action — no approval needed
                match self
                    .ax_element_at_point(coords.resolved_x, coords.resolved_y)
                    .await
                {
                    Ok(info) => ToolResult {
                        success: true,
                        output: serde_json::to_string_pretty(&json!({
                            "point": Self::resolved_coordinates_json(&coords),
                            "element": info,
                        }))
                        .unwrap_or_default(),
                        error: None,
                    },
                    Err(e) => ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "Hit test failed at ({}, {}): {e}",
                            coords.resolved_x, coords.resolved_y
                        )),
                    },
                }
            }
            "inspect_window_elements" => {
                let query = match Self::parse_front_window_query(&args) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                let max_results = match Self::parse_max_results_arg(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        });
                    }
                };
                match self.front_window_element_match_report(&query, max_results).await {
                    Ok(report) => ToolResult {
                        success: true,
                        output: serde_json::to_string_pretty(&report).unwrap_or_default(),
                        error: None,
                    },
                    Err(error) => ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to inspect front window elements: {error}")),
                    },
                }
            }
            other => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Unsupported action '{other}'. Allowed: launch_app, activate_app, run_applescript, move_mouse, click_at, read_focused_element, hit_test, inspect_window_elements"
                    )),
                })
            }
        };

        // ── Determine if this was a mutating action ──
        let is_mutating = matches!(action, "move_mouse" | "click_at")
            || (action == "run_applescript" && Self::is_mutating_applescript(&args));

        // If no expectations: mutating actions soft-fail with ambiguous status,
        // read-only actions return raw result as before.
        if expectations.is_empty() {
            if is_mutating && raw_result.success {
                return Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&json!({
                        "execution": parse_tool_output(&raw_result.output),
                        "verification_status": "ambiguous",
                        "message": format!(
                            "No 'expect' was provided for mutating action '{action}'. The input event was dispatched, but resulting UI state was not verified."
                        ),
                    }))
                    .unwrap_or_default(),
                    error: None,
                });
            }
            return Ok(raw_result);
        }

        // ── GUI verification: build post-action evidence and verify ──
        let base_evidence = parse_tool_output(&raw_result.output);

        // Apply backend-native wait strategy (AccessibilityEvent) before
        // collecting post-action evidence.
        if let WaitStrategy::AccessibilityEvent {
            notification,
            timeout_ms,
        } = &wait_strategy
        {
            if raw_result.success {
                if let Err(error) = self
                    .apply_accessibility_event_wait(notification, *timeout_ms)
                    .await
                {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(error.to_string()),
                    });
                }
            }
        }

        let post_keys = gui_verify::infer_evidence_keys(&expectations);
        // Only pass FixedMs/PollUntilVerified to the generic wait handler;
        // AccessibilityEvent was already handled above.
        let runtime_wait_strategy = match &wait_strategy {
            WaitStrategy::FixedMs { .. } | WaitStrategy::PollUntilVerified { .. } => {
                wait_strategy.clone()
            }
            _ => WaitStrategy::None,
        };
        let post_evidence = if raw_result.success {
            match gui_verify::apply_wait_strategy(
                &runtime_wait_strategy,
                || async {
                    let mut evidence = base_evidence.clone();
                    let extra = self.collect_gui_evidence(&post_keys).await?;
                    merge_json_objects(&mut evidence, &extra);
                    Ok(evidence)
                },
                &expectations,
            )
            .await
            {
                Ok(evidence) => evidence,
                Err(error) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(error.to_string()),
                    });
                }
            }
        } else {
            base_evidence
        };

        let post_obs = gui_verify::observation("osascript_output", post_evidence.clone());
        let report = gui_verify::build_report(
            raw_result.success,
            pre_observation,
            Some(post_obs),
            &expectations,
            &post_evidence,
        );

        Ok(ToolResult::from_gui_report(&report))
    }
}

fn parse_tool_output(output: &str) -> Value {
    serde_json::from_str(output).unwrap_or_else(|_| json!({"output": output}))
}

fn merge_json_objects(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(key) {
                    Some(existing) => merge_json_objects(existing, value),
                    None => {
                        target_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target_slot, source_value) => {
            *target_slot = source_value.clone();
        }
    }
}

fn default_mac_approval_keys() -> Vec<String> {
    vec!["title".into(), "dialog_present".into()]
}

fn summarize_mac_action(action: &str, args: &Value) -> String {
    match action {
        "launch_app" | "activate_app" => args
            .get("app_name")
            .and_then(Value::as_str)
            .map(|app_name| format!("{action} '{app_name}'"))
            .unwrap_or_else(|| action.to_string()),
        "run_applescript" => "run AppleScript".into(),
        "move_mouse" => {
            let x = args.get("x").and_then(Value::as_i64).unwrap_or(0);
            let y = args.get("y").and_then(Value::as_i64).unwrap_or(0);
            format!("move mouse to ({x}, {y})")
        }
        "click_at" => {
            let x = args.get("x").and_then(Value::as_i64).unwrap_or(0);
            let y = args.get("y").and_then(Value::as_i64).unwrap_or(0);
            let modifiers = args
                .get("keys")
                .and_then(Value::as_array)
                .map(|keys| {
                    keys.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("+")
                })
                .filter(|joined| !joined.is_empty());
            match modifiers {
                Some(keys) => format!("click at ({x}, {y}) with {keys}"),
                None => format!("click at ({x}, {y})"),
            }
        }
        "read_focused_element" => "read focused element".into(),
        "hit_test" => {
            let x = args.get("x").and_then(Value::as_i64).unwrap_or(0);
            let y = args.get("y").and_then(Value::as_i64).unwrap_or(0);
            format!("hit test at ({x}, {y})")
        }
        "inspect_window_elements" => "inspect front window accessibility elements".into(),
        _ => action.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AutonomyLevel, SecurityPolicy};

    fn test_security(autonomy: AutonomyLevel) -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy,
            workspace_dir: std::env::temp_dir(),
            ..SecurityPolicy::default()
        })
    }

    #[test]
    fn mac_automation_tool_name() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        assert_eq!(tool.name(), "mac_automation");
    }

    #[test]
    fn mac_automation_schema_has_actions() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["action"].is_object());
        assert!(schema["properties"]["app_name"].is_object());
        assert!(schema["properties"]["script"].is_object());
    }

    #[tokio::test]
    async fn mac_automation_blocks_in_readonly() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::ReadOnly));
        let result = tool
            .execute(json!({"action":"launch_app","app_name":"MongoDB Compass"}))
            .await
            .expect("readonly execution should return a structured tool result");
        assert!(!result.success);
        assert!(result.error.unwrap_or_default().contains("read-only"));
    }

    #[tokio::test]
    async fn mac_automation_requires_script_for_applescript_action() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let result = tool
            .execute(json!({"action":"run_applescript"}))
            .await
            .expect("missing script should return a structured tool result");
        assert!(!result.success);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Provide either 'script' or 'script_lines'"));
    }

    #[tokio::test]
    async fn mac_automation_click_at_requires_coordinates() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let result = tool
            .execute(json!({"action":"click_at"}))
            .await
            .expect("missing coords should return a structured tool result");
        assert!(!result.success);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Missing 'x' coordinate"));
    }

    #[tokio::test]
    async fn mac_automation_move_mouse_requires_coordinates() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let result = tool
            .execute(json!({"action":"move_mouse"}))
            .await
            .expect("missing coords should return a structured tool result");
        assert!(!result.success);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Missing 'x' coordinate"));
    }

    #[tokio::test]
    async fn mac_automation_hit_test_requires_coordinates() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let result = tool
            .execute(json!({"action":"hit_test"}))
            .await
            .expect("missing coords should return a structured tool result");
        assert!(!result.success);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Missing 'x' coordinate"));
    }

    #[test]
    fn mac_automation_schema_includes_new_actions() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let schema = tool.parameters_schema();
        let actions = schema["properties"]["action"]["enum"]
            .as_array()
            .expect("action enum should be an array");
        let action_strs: Vec<&str> = actions.iter().filter_map(|v| v.as_str()).collect();
        assert!(action_strs.contains(&"move_mouse"));
        assert!(action_strs.contains(&"click_at"));
        assert!(action_strs.contains(&"read_focused_element"));
        assert!(action_strs.contains(&"hit_test"));
        assert!(action_strs.contains(&"inspect_window_elements"));
        assert!(schema["properties"]["x"].is_object());
        assert!(schema["properties"]["y"].is_object());
        assert!(schema["properties"]["keys"].is_object());
        assert!(schema["properties"]["coordinate_space"].is_object());
    }

    #[test]
    fn parse_modifier_keys_normalizes_aliases() {
        let keys =
            MacAutomationTool::parse_modifier_keys(&json!({"keys": ["alt", "shift", "cmd"]}))
                .unwrap();
        assert_eq!(keys, vec!["option", "shift", "command"]);
    }

    #[test]
    fn front_window_query_roundtrips_through_key_encoding() {
        let key = gui_verify::encode_front_window_match_count_key(
            Some("AXImage"),
            Some("Recent"),
            Some("thumbnail"),
            None,
        );
        let query = MacAutomationTool::front_window_query_from_key(&key).unwrap();
        assert_eq!(query.role.as_deref(), Some("AXImage"));
        assert_eq!(query.title_contains.as_deref(), Some("Recent"));
        assert_eq!(query.description_contains.as_deref(), Some("thumbnail"));
        assert!(query.value_contains.is_none());
    }

    #[test]
    fn front_window_query_script_serializes_row_contents() {
        let script = MacAutomationTool::front_window_query_script(
            &FrontWindowElementQuery {
                role: Some("AXImage".into()),
                title_contains: None,
                description_contains: Some("thumbnail".into()),
                value_contains: None,
            },
            3,
        );

        assert!(script.contains("set end of outputRows to contents of rowText"));
    }

    #[tokio::test]
    async fn resolve_coordinates_without_coordinate_space_passthrough() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let resolved = tool
            .resolve_coordinates(&json!({"x": 100, "y": 200}), "click_at")
            .await
            .unwrap();
        assert_eq!(resolved.requested_x, 100);
        assert_eq!(resolved.requested_y, 200);
        assert_eq!(resolved.resolved_x, 100);
        assert_eq!(resolved.resolved_y, 200);
        assert!(resolved.coordinate_space.is_none());
    }

    #[test]
    fn is_mutating_applescript_detects_click() {
        assert!(MacAutomationTool::is_mutating_applescript(
            &json!({"script": "tell application \"System Events\" to click button 1"})
        ));
    }

    #[test]
    fn is_mutating_applescript_detects_keystroke() {
        assert!(MacAutomationTool::is_mutating_applescript(
            &json!({"script_lines": ["tell application \"System Events\"", "keystroke return"]})
        ));
    }

    #[test]
    fn is_mutating_applescript_ignores_read_only() {
        assert!(!MacAutomationTool::is_mutating_applescript(
            &json!({"script": "tell application \"System Events\" to return name of first process whose frontmost is true"})
        ));
    }

    #[test]
    fn adds_hint_for_applescript_ui_lookup_failures() {
        let result = MacAutomationTool::add_applescript_failure_hint(ToolResult {
            success: false,
            output: String::new(),
            error: Some("System Events got an error: Can't get button 1 of window 1".into()),
        });

        let error = result.error.unwrap_or_default();
        assert!(error.contains("Can't get button 1 of window 1"));
        assert!(error.contains("retry with mac_automation action=click_at"));
    }

    #[test]
    fn does_not_add_hint_for_non_ui_applescript_failures() {
        let result = MacAutomationTool::add_applescript_failure_hint(ToolResult {
            success: false,
            output: String::new(),
            error: Some("Expected end of line but found identifier".into()),
        });

        assert_eq!(
            result.error.as_deref(),
            Some("Expected end of line but found identifier")
        );
    }

    #[test]
    fn summarize_click_at_includes_coords() {
        let summary = summarize_mac_action("click_at", &json!({"x": 100, "y": 200}));
        assert_eq!(summary, "click at (100, 200)");
    }

    #[test]
    fn summarize_click_at_includes_modifier_keys() {
        let summary = summarize_mac_action(
            "click_at",
            &json!({"x": 100, "y": 200, "keys": ["option", "shift"]}),
        );
        assert_eq!(summary, "click at (100, 200) with option+shift");
    }

    #[test]
    fn summarize_move_mouse_includes_coords() {
        let summary = summarize_mac_action("move_mouse", &json!({"x": 100, "y": 200}));
        assert_eq!(summary, "move mouse to (100, 200)");
    }

    #[test]
    fn summarize_hit_test_includes_coords() {
        let summary = summarize_mac_action("hit_test", &json!({"x": 50, "y": 75}));
        assert_eq!(summary, "hit test at (50, 75)");
    }
}
