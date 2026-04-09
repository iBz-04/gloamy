# chat.db forensics (read-only)

Use SQL for history/analytics; JXA chat iteration is slow and often redacted.

## Permissions
- Requires Full Disk Access for `~/Library/Messages/chat.db`.
- Keep operations read-only; editing DB risks corruption and legal/privacy issues.

## Schema snapshot
- `message`: body (`text` often NULL on modern macOS), `attributedBody` (typedstream BLOB), `date` (ns since 2001-01-01), `is_from_me`, `handle_id`.
- `handle`: `id` = phone/email.
- `chat`: `guid` maps to JXA `chat.id`.
- Joins: `chat_message_join` and `chat_handle_join`.

## Quick pull (last 5 from handle)
```javascript
const q = `
SELECT text,
  datetime((date / 1000000000) + 978307200, 'unixepoch', 'localtime') AS sent_at
FROM message
JOIN handle ON message.handle_id = handle.rowid
WHERE handle.id = '+15551234567'
ORDER BY date DESC LIMIT 5;
`;
const raw = Application.currentApplication().doShellScript(
  `sqlite3 -json ~/Library/Messages/chat.db "${q}"`
);
const msgs = JSON.parse(raw);
```
- Use `-json` to avoid manual parsing.
- Convert time via `(date/1e9) + 978307200` (Cocoa epoch → Unix).

## Typedstream bodies
- Rich/edited messages store content in `attributedBody` (NSAttributedString archive).
- JXA cannot decode; call out to a helper (Python or Rust) and parse JSON output.
- Popular CLI: `imessage-exporter` (`/usr/local/bin/imessage-exporter -f json -o /tmp/messages_export`), then read JSON in JXA.

## Safety and verification
- Always copy the DB before experiments; avoid writes.
- Log SQL errors to a file to diagnose permissions vs schema drift.
- Expect schema changes between macOS releases; guard queries accordingly.
