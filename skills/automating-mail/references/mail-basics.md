# Mail Automation Basics

This document shows equivalent patterns in JXA (JavaScript for Automation), PyXA (Python for Apple Automation), and PyObjC (Python-Objective-C bridge) for basic Mail.app operations.

## Bootstrapping

### JXA
```javascript
const Mail = Application("Mail");
Mail.activate();
```

### PyXA
```python
import PyXA
mail = PyXA.Application("Mail")
mail.activate()
```

### PyObjC
```python
from objc import *
import Mail
mail_app = Mail.sharedApplication()
mail_app.activate()
```

## Selected Messages

### JXA
```javascript
const sel = Mail.selection();
const subjects = sel.map(m => m.subject());
```

### PyXA
```python
selection = mail.selection()
subjects = [msg.subject for msg in selection]
```

### PyObjC
```python
selection = mail_app.selection()
subjects = [msg.subject() for msg in selection]
```

## Open Mailbox by Name

### JXA
```javascript
const acct = Mail.accounts.byName("iCloud");
const inbox = acct.mailboxes.byName("INBOX");
```

### PyXA
```python
account = mail.accounts().byName("iCloud")
inbox = account.mailboxes().byName("INBOX")
```

### PyObjC
```python
accounts = mail_app.accounts()
account = next(acc for acc in accounts if acc.displayName() == "iCloud")
mailboxes = account.mailboxes()
inbox = next(mb for mb in mailboxes if mb.name() == "INBOX")
```

