# Shell integration pitfalls (AppleScript)

## PATH is minimal in `do shell script`
- `/bin/sh` non-interactive, non-login.
- Default PATH: `/usr/bin:/bin:/usr/sbin:/sbin`.

## Fixes
- Use absolute paths (most reliable).
- Or load system PATH helper:
```applescript
do shell script "eval $(/usr/libexec/path_helper -s); git status"
```

## Quoting rule
- Always use `quoted form of` for variables.

