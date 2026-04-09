# Tooling and workflow (VS Code + build tasks)

## Source format
- Keep scripts as `.applescript` (text) in git.
- Compile to `.scpt` or `.app` only for distribution.

## VS Code build task (osacompile)
```json
{
  "label": "Compile AppleScript",
  "type": "shell",
  "command": "osacompile -o build/main.scpt source/main.applescript",
  "group": "build"
}
```

## Diff strategy
- Prefer text sources for review and diffs.
- If binary scripts are unavoidable, generate compiled artifacts in CI, not source control.

