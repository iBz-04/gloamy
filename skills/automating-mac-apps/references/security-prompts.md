# Automation security prompts (TCC overview)

## What triggers prompts
- Apple Events between apps (Automation permission).
- UI scripting via System Events (Accessibility permission).

## Practical guidance
- Expect a one-time prompt per runner app (Terminal, Script Editor, .app).
- Approve prompts during interactive development.
- In CI/headless, assume prompts cannot be approved.

## Safe handling
- Detect failures and return clear errors.
- Provide a fallback path or exit early.
- Use MDM profiles for enterprise automation.

