# CI/CD and TCC (safe guidance)

## Core constraints
- Headless runners cannot approve Automation or Accessibility prompts.
- UI scripting is effectively blocked in CI unless pre-authorized.
- Prefer dictionary automation and shell commands in CI.

## Safe practices
- Use dry-run mode first; emit JSON reports to stdout.
- Fail gracefully when permissions are missing.
- Use MDM profiles to pre-authorize Apple Events and Accessibility in enterprise environments.

## Avoid (unsafe / brittle)
- Direct TCC database edits or sqlite injection.
- Clicking security prompts via UI scripting.

## Headless check pattern (AppleScript)
```applescript
try
  tell application "System Events" to set uiEnabled to UI elements enabled
on error
  -- Likely headless
end try
```

