# HTML + signature workflow

## Pattern
1) Create message with `visible: false`.
2) Set `htmlContent`.
3) Make visible and set signature via UI scripting.

```javascript
const Mail = Application("Mail");
const msg = Mail.OutgoingMessage({ subject: "Update", visible: false });
Mail.outgoingMessages.push(msg);
msg.htmlContent = "<html><body><h1>Status</h1></body></html>";
msg.visible = true;

// UI scripting signature selection (outline)
const se = Application("System Events");
const proc = se.processes.byName("Mail");
// find signature popup button, select signature
```

