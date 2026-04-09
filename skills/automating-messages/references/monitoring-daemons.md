# Monitoring + daemon pattern

Use when you need bot-like behavior after AppleScript event handlers were removed.

## Polling design
1) Track the latest `message.rowid` in a state file (e.g., `~/.imessage_bot_last_id`).
2) On each run, query `chat.db` for `rowid > last_id`.
3) Parse results and dispatch actions (reply, log, trigger script).
4) Persist the new max `rowid`.

Example shell (invoked from JXA via `doShellScript`):
```bash
STATE=~/.imessage_bot_last_id
[ -f "$STATE" ] || echo 0 > "$STATE"
LAST=$(cat "$STATE")
SQL="SELECT rowid, handle.id, text FROM message JOIN handle ON handle.rowid = message.handle_id WHERE rowid > $LAST ORDER BY rowid;"
RESULTS=$(sqlite3 -json ~/Library/Messages/chat.db "$SQL")
echo "$RESULTS"
NEW_MAX=$(echo "$RESULTS" | jq '.[].rowid' | sort -n | tail -1)
[ -n "$NEW_MAX" ] && echo "$NEW_MAX" > "$STATE"
```

## launchd setup (preferred over while-true loops)
- Create `~/Library/LaunchAgents/com.user.messagebot.plist` with `ProgramArguments` pointing to your script and `StartInterval` (e.g., 10 seconds).
- Load/unload with `launchctl bootstrap gui/$UID ...` and `launchctl bootout gui/$UID ...`.
- Keep scripts idempotent; `launchd` may overlap if runs take long.

## Hardening
- Verify Full Disk Access for the agent binary/script.
- Add structured logging (JSON) for postmortem analysis.
- Rate-limit actions to avoid UI thrash when many messages arrive.
