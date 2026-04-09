# Mailbox archiver

## Archive messages older than N days
```javascript
const Mail = Application("Mail");
const account = Mail.accounts.byName("iCloud");
const archive = account.mailboxes.byName("Archive");
const cutoff = new Date();
cutoff.setDate(cutoff.getDate() - 30);

account.mailboxes().forEach(box => {
  try {
    const msgs = box.messages();
    msgs.forEach(m => {
      const received = m.dateReceived();
      if (received && received < cutoff) {
        Mail.move(m, { to: archive });
      }
    });
  } catch (e) {}
});
```

Notes:
- Batch move lists when possible for performance.
- Skip special mailboxes (Trash, Junk) as needed.

