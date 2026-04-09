# Mail JXA recipes

## Filter unread flagged

**JXA:**
```javascript
const msgs = Mail.inbox.messages.whose({ flaggedStatus: true, readStatus: false });
```

**PyXA:**
```python
import PyXA

mail = PyXA.Application("Mail")
inbox = mail.inboxes()[0]

# Filter unread flagged messages
unread_flagged = inbox.messages().filter(
    lambda msg: msg.flagged and not msg.read_status
)

print(f"Found {len(unread_flagged)} unread flagged messages")
```

## Move messages (batch)

**JXA:**
```javascript
const archive = Mail.accounts.byName("iCloud").mailboxes.byName("Archive");
Mail.move(msgs, { to: archive });
```

**PyXA:**
```python
import PyXA

mail = PyXA.Application("Mail")

# Get archive mailbox
icloud_account = mail.accounts().by_name("iCloud")
archive_mailbox = icloud_account.mailboxes().by_name("Archive")

# Move messages to archive
# Note: PyXA may require individual moves for reliability
for message in unread_flagged:
    message.move_to(archive_mailbox)

print(f"Moved {len(unread_flagged)} messages to archive")
```

## Compose message

**JXA:**
```javascript
const msg = Mail.OutgoingMessage({ subject: "Report", content: "See attached", visible: true });
Mail.outgoingMessages.push(msg);
msg.toRecipients.push(Mail.Recipient({ address: "client@example.com" }));
```

**PyXA:**
```python
import PyXA

mail = PyXA.Application("Mail")

# Create outgoing message
message = mail.outgoing_messages().push({
    "subject": "Report",
    "content": "See attached",
    "to_recipients": ["client@example.com"],
    "visible": True
})

print("Message composed and ready for sending")
```

## Attach file

**JXA:**
```javascript
msg.content.attachments.push(Mail.Attachment({ fileName: Path("/Users/you/report.pdf") }));
```

**PyXA:**
```python
import PyXA

# Attach file to the message we just created
attachment_path = "/Users/you/report.pdf"
message.attachments().push({
    "file_name": attachment_path
})

print(f"Attached file: {attachment_path}")
```

