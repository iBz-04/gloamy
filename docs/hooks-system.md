# Hooks System

Lifecycle hooks for extending Gloamy behavior at key execution points.

Last verified: **April 8, 2026**.

## Overview

The hooks system allows you to intercept and modify runtime events through custom or built-in hook handlers. Hooks run in-process with the same privileges as the main runtime.

## Configuration

```toml
[hooks]
enabled = true

[hooks.builtin]
command_logger = true
```

### Configuration Options

| Key | Default | Description |
|-----|---------|-------------|
| `hooks.enabled` | `true` | Enable hook system execution |
| `hooks.builtin.command_logger` | `false` | Log all tool calls with timing and results |

## Hook Types

### Void Hooks (Fire-and-Forget)

These hooks run in parallel and cannot modify or cancel operations. Use for logging, metrics, and side effects.

| Hook | Trigger | Arguments |
|------|---------|-----------|
| `on_gateway_start` | Gateway server starts | `host`, `port` |
| `on_gateway_stop` | Gateway server stops | - |
| `on_session_start` | User session begins | `session_id`, `channel` |
| `on_session_end` | User session ends | `session_id`, `channel` |
| `on_llm_input` | Before LLM call | `messages`, `model` |
| `on_llm_output` | After LLM response | `response` |
| `on_after_tool_call` | After tool execution | `tool`, `result`, `duration` |
| `on_message_sent` | After sending message | `channel`, `recipient`, `content` |
| `on_heartbeat_tick` | Periodic heartbeat | - |

### Modifying Hooks (Sequential, Can Cancel)

These hooks run sequentially by priority. Each hook receives the output of the previous hook. Hooks can modify data or cancel operations entirely.

| Hook | Trigger | Can Modify | Can Cancel |
|------|---------|------------|------------|
| `before_model_resolve` | Before model ID resolved | Provider/model | Yes |
| `before_prompt_build` | Before prompt assembled | Prompt text | Yes |
| `before_llm_call` | Before LLM API call | Messages, model | Yes |
| `before_tool_call` | Before tool execution | Tool name, args | Yes |
| `on_message_received` | When message received | Message | Yes |
| `on_message_sending` | Before sending reply | Channel, recipient, content | Yes |

## Built-in Hooks

### Command Logger

Logs all tool calls with timing information for auditing.

```toml
[hooks]
enabled = true

[hooks.builtin]
command_logger = true
```

**Log Format:**

```
[14:32:15] shell (45ms) success=true
[14:32:20] file_read (12ms) success=false reason=File not found
```

**Use Cases:**
- Security auditing
- Performance monitoring
- Debugging tool failures

## Creating Custom Hooks

Implement the `HookHandler` trait:

```rust
use async_trait::async_trait;
use gloamy::hooks::{HookHandler, HookResult};
use gloamy::tools::traits::ToolResult;
use std::time::Duration;

pub struct MyCustomHook;

#[async_trait]
impl HookHandler for MyCustomHook {
    fn name(&self) -> &str {
        "my-custom-hook"
    }
    
    fn priority(&self) -> i32 {
        10  // Higher priority hooks run first
    }
    
    // Example: Log slow tool calls
    async fn on_after_tool_call(&self, tool: &str, result: &ToolResult, duration: Duration) {
        if duration.as_secs() > 5 {
            tracing::warn!("Slow tool call: {} took {}s", tool, duration.as_secs());
        }
    }
    
    // Example: Block specific tools
    async fn before_tool_call(&self, name: String, args: serde_json::Value) -> HookResult<(String, Value)> {
        if name == "shell" {
            if let Some(cmd) = args.get("cmd").and_then(|v| v.as_str()) {
                if cmd.contains("rm -rf /") {
                    return HookResult::Cancel("Destructive command blocked".into());
                }
            }
        }
        HookResult::Continue((name, args))
    }
}
```

### Hook Priority

| Priority | Execution Order |
|----------|-----------------|
| Higher positive values | Execute first |
| `0` | Default priority |
| Lower negative values | Execute last |

### Hook Results

```rust
// Continue with (possibly modified) data
HookResult::Continue(data)

// Cancel operation with reason
HookResult::Cancel("Operation blocked by policy".into())
```

## Security Considerations

- Hooks run in-process with full runtime privileges
- Keep hook handlers narrowly scoped and auditable
- Cancel operations only when justified
- Avoid blocking operations in void hooks
- Validate all modifications in modifying hooks

## Integration with Agent System

Hooks complement the agent system by:

- **Pre-processing**: Modify prompts before LLM calls
- **Filtering**: Block unwanted tools or messages
- **Logging**: Audit all operations for compliance
- **Enrichment**: Add context to messages
- **Rate limiting**: Throttle operations per user

## Debugging Hooks

Enable debug logging for hook execution:

```bash
RUST_LOG=gloamy::hooks=debug gloamy daemon
```

Common log patterns:

| Pattern | Meaning |
|---------|---------|
| `Hook {} executing` | Hook handler started |
| `Hook {} cancelled operation` | Modifying hook returned Cancel |
| `Hook {} completed in {}ms` | Hook execution timing |

## Examples

### Block Dangerous Shell Commands

```rust
async fn before_tool_call(&self, name: String, args: Value) -> HookResult<(String, Value)> {
    if name == "shell" {
        let cmd = args.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
        let forbidden = ["rm -rf /", ":(){ :|:& };:", "mkfs"];
        for pattern in &forbidden {
            if cmd.contains(pattern) {
                return HookResult::Cancel(format!("Forbidden pattern: {}", pattern));
            }
        }
    }
    HookResult::Continue((name, args))
}
```

### Add Custom Headers to Outbound Messages

```rust
async fn on_message_sending(&self, channel: String, recipient: String, content: String) 
    -> HookResult<(String, String, String)> {
    let tagged = format!("[via Gloamy] {}", content);
    HookResult::Continue((channel, recipient, tagged))
}
```

### Log LLM Interactions

```rust
async fn on_llm_input(&self, messages: &[ChatMessage], model: &str) {
    tracing::info!("LLM request to {}: {} messages", model, messages.len());
}

async fn on_llm_output(&self, response: &ChatResponse) {
    tracing::info!("LLM response: {} tokens", response.tokens_used);
}
```

## Related Documentation

- [config-reference.md](config-reference.md) - Hook configuration options
- [security-roadmap.md](security-roadmap.md) - Security considerations
- [audit-logging.md](audit-logging.md) - Audit and compliance
