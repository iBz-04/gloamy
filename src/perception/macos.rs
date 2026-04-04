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
        let script = r#"
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
    if (depth > 8) return null; // Limit recursion depth to prevent hangs
    
    var role = ""; try { role = elem.role(); } catch(e) {}
    if (!role) role = "unknown";

    var name = null; try { var rawName = elem.name(); if (typeof rawName === 'string' && rawName.length > 0) name = rawName; } catch(e) {}
    var value = null; try { var val = elem.value(); if ((typeof val === 'string' && val.length > 0) || typeof val === 'number') value = val.toString(); } catch(e) {}
    
    var bounds = getBounds(elem);
    
    var children = [];
    try {
        var uiElements = elem.uiElements();
        if (uiElements) {
            for (var i = 0; i < uiElements.length; i++) {
                var childNode = dumpNode(uiElements[i], depth + 1);
                if (childNode) children.push(childNode);
            }
        }
    } catch(e) {}
    
    // Generate a pseudo-ID based on role and name
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
                return JSON.stringify(dumpNode(win, 0));
            }
        } catch(e) {}
    }
    return JSON.stringify(null);
}
run();
"#;

        let result = tokio::time::timeout(
            Duration::from_secs(15),
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
