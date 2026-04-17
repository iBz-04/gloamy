use crate::perception::traits::{PerceptionProvider, ScreenState, WidgetNode};
use async_trait::async_trait;
use std::time::Duration;

/// A perception provider that extracts the macOS accessibility tree via JXA (JavaScript for Automation).
pub struct MacOsPerceptionProvider;

impl MacOsPerceptionProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsPerceptionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PerceptionProvider for MacOsPerceptionProvider {
    fn name(&self) -> &str {
        "macos_a11y"
    }

    async fn capture_state(&self) -> anyhow::Result<ScreenState> {
        // Budgeted widget-tree capture. Every `elem.role()` / `elem.name()` /
        // `elem.uiElements()` call crosses the JXA ↔ AX bridge, so a naïve
        // full-depth dump of a Microsoft Office window can take 10+ seconds.
        //
        // We cap three axes to keep p95 capture under ~3s:
        //   - recursion depth      (MAX_DEPTH)
        //   - total node count     (MAX_NODES)
        //   - wall-clock budget    (TIME_BUDGET_MS)
        //
        // When any limit is reached we return the partial tree with a
        // `truncated: true` flag on the affected node so downstream code can
        // still ground coordinates and the preflight gate still sees a valid
        // widget_tree. Invisible / zero-size elements are skipped entirely
        // since they cannot be clicked.
        let script = r#"
var LIMITS = { maxDepth: 5, maxNodes: 400, timeBudgetMs: 2500 };
var START = Date.now();
var STATE = { count: 0, truncated: false };

function budgetExceeded() {
    return STATE.count >= LIMITS.maxNodes
        || (Date.now() - START) > LIMITS.timeBudgetMs;
}

function getBounds(elem) {
    try {
        var pos = elem.position();
        var size = elem.size();
        if (pos && size) {
            return {
                x: Math.round(pos[0]),
                y: Math.round(pos[1]),
                width: Math.round(size[0]),
                height: Math.round(size[1])
            };
        }
    } catch(e) {}
    return null;
}

function dumpNode(elem, depth) {
    if (depth > LIMITS.maxDepth) { STATE.truncated = true; return null; }
    if (budgetExceeded()) { STATE.truncated = true; return null; }

    var role = ""; try { role = elem.role(); } catch(e) {}
    if (!role) role = "unknown";

    var name = null;
    try {
        var rawName = elem.name();
        if (typeof rawName === 'string' && rawName.length > 0) name = rawName;
    } catch(e) {}

    var value = null;
    try {
        var val = elem.value();
        if ((typeof val === 'string' && val.length > 0) || typeof val === 'number') {
            value = val.toString();
        }
    } catch(e) {}

    var bounds = getBounds(elem);

    // Skip invisible / zero-size elements — they cannot be clicked and only
    // inflate the traversal budget.
    if (bounds && (bounds.width <= 0 || bounds.height <= 0) && depth > 0) {
        return null;
    }

    STATE.count += 1;

    var children = [];
    if (!budgetExceeded()) {
        try {
            var uiElements = elem.uiElements();
            if (uiElements) {
                for (var i = 0; i < uiElements.length; i++) {
                    if (budgetExceeded()) { STATE.truncated = true; break; }
                    var childNode = dumpNode(uiElements[i], depth + 1);
                    if (childNode) children.push(childNode);
                }
            }
        } catch(e) {}
    } else {
        STATE.truncated = true;
    }

    var id = role;
    if (name) { id += "_" + name.replace(/[^a-zA-Z0-9]/g, "_").substring(0, 20); }

    return {
        id: id,
        role: role,
        name: name,
        value: value,
        bounds: bounds,
        children: children
    };
}

function run() {
    var se = Application('System Events');
    var frontProcs = se.processes.whose({ frontmost: true });
    if (frontProcs.length > 0) {
        var frontProc = frontProcs[0];
        try {
            var windows = frontProc.windows;
            if (windows && windows.length > 0) {
                var win = windows[0];
                var tree = dumpNode(win, 0);
                if (tree && STATE.truncated) { tree.truncated = true; }
                return JSON.stringify(tree);
            }
        } catch(e) {}
    }
    return JSON.stringify(null);
}
run();
"#;

        // Hard outer timeout slightly above the in-script budget to allow for
        // osascript startup (~300ms) + JXA compilation. If this fires, the
        // in-script budget already failed to bail, which is a bug worth
        // surfacing.
        let result = tokio::time::timeout(
            Duration::from_secs(6),
            tokio::process::Command::new("osascript")
                .arg("-l")
                .arg("JavaScript")
                .arg("-e")
                .arg(script.trim())
                .output(),
        )
        .await?;

        let output = result?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("osascript failed: {}", stderr);
        }

        let json_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if json_str == "null" || json_str.is_empty() {
            return Ok(ScreenState {
                screenshot_path: None,
                widget_tree: None,
                extracted_text: vec![],
            });
        }

        let widget_tree: WidgetNode = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse accessibility JSON: {}", e))?;

        Ok(ScreenState {
            screenshot_path: None,
            widget_tree: Some(widget_tree),
            extracted_text: vec![],
        })
    }
}
