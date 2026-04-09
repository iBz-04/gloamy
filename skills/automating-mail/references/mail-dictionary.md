# Mail dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
selection                            Mail.selection()
subject of message                   msg.subject()
flagged status                       msg.flaggedStatus()
read status                          msg.readStatus()
mailbox "INBOX"                      account.mailboxes.byName("INBOX")
make new outgoing message            Mail.OutgoingMessage({ ... }) -> push()
```

Notes:
- Use batch reads on collections (messages.subject()).
- Use Path() for attachment file names.

