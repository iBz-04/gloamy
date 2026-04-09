# JXA Mail recipes

## Send email with attachments
```javascript
const mail = Application("Mail");
const msg = mail.OutgoingMessage().make();
msg.visible = true;
msg.toRecipients.push(mail.Recipient({ address: "recipient@example.com" }));
msg.subject = "Report";
msg.content = "See attached.";
msg.attachments.push(mail.Attachment({ fileName: Path("/path/to/file.pdf") }));
msg.send();
JSON.stringify({ success: true });
```

## Export selected messages
```javascript
const mail = Application("Mail");
const sel = mail.selection();
const out = sel.length
  ? sel.map(m => ({ subject: m.subject(), sender: m.sender() }))
  : [];
JSON.stringify({ count: out.length, messages: out });
```
