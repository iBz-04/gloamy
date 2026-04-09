# Notes Advanced (HTML, attachments, ObjC/UI)

## HTML body rules
- Use supported tags: `<h1>-<h3>`, `<b>`, `<i>`, `<u>`, `<ul>/<ol>/<li>`, `<div>/<p>/<br>`, `<a>`.
- Notes wraps paragraphs in `<div>`; keep bodies well-formed. Avoid `<script>`/`<style>` (stripped).

## Append content safely
```javascript
function appendBody(note, html) {
  const body = note.body();
  const replacement = body.includes("</body>") ?
    body.replace("</body>", `${html}</body>`) :
    body + html;
  note.body = replacement;
}
```

## Attachments / images workaround
- Direct attachment creation via JXA is limited. Use clipboard + UI scripting:
  1) Load file/image to NSPasteboard via ObjC.
  2) Activate Notes, focus note, send Cmd+V via System Events.
- Requires Accessibility + FDA permissions.

## JSON import/export
- Import: read JSON via `JSON.parse` and create folders/notes to mirror structure.
- Export: iterate notes, collect `{name, body, creationDate, id}`, and write to file via ObjC `NSString`/`NSFileManager`.

## Move and account boundaries
- Use `Notes.move(spec, { to: folder })`. Crossing accounts effectively copies and may change IDs.

## Performance
- Use `.whose` to filter server-side; avoid property access in tight loops.
- Batch deletes/moves by calling commands on specifiers.

## Error handling
- -1728 (can't get) → invalid specifier; check existence.
- -1700 (type mismatch) → wrong type; ensure strings/dates are correct.
- -10000 (handler failed) → retry or restart Notes; check permissions.
