# Attachment extraction pipeline

## Save attachments from selected messages
```javascript
const Mail = Application("Mail");
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const outDir = "/Users/you/Downloads/attachments";

const sel = Mail.selection();
sel.forEach(m => {
  m.mailAttachments().forEach(a => {
    const path = outDir + "/" + a.name();
    Mail.save(a, { in: path });
  });
});
```

Notes:
- Prefer a user-writable directory (Documents/Downloads).
- For errors, fall back to shell copy if the attachment provides a fileName path.

