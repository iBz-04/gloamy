# Mail JXA advanced patterns

## Batch property reads

**JXA:**
```javascript
const msgs = Mail.inbox.messages;
const ids = msgs.id();
const dates = msgs.dateReceived();
```

**PyXA:**
```python
import PyXA

mail = PyXA.Application("Mail")
inbox = mail.inboxes()[0]  # Get first inbox
messages = inbox.messages()

# Batch read properties (efficient)
ids = messages.id()
dates = messages.date_received()
subjects = messages.subject()
```

## HTML composition (invisible)

**JXA:**
```javascript
const htmlMsg = Mail.OutgoingMessage({ subject: "HTML", visible: false });
Mail.outgoingMessages.push(htmlMsg);
htmlMsg.htmlContent = "<html><body><h1>Status</h1></body></html>";
htmlMsg.visible = true;
```

**PyXA:**
```python
import PyXA

mail = PyXA.Application("Mail")

# Create HTML message (invisible initially)
html_message = mail.outgoing_messages().push({
    "subject": "HTML Status Report",
    "content": "",  # Will be replaced with HTML
    "visible": False
})

# Set HTML content
html_content = "<html><body><h1>Status Report</h1><p>All systems operational.</p></body></html>"
html_message.content = html_content

# Make visible for editing/sending
html_message.visible = True
```

## Signature via UI scripting (outline)

**JXA:**
```javascript
const se = Application("System Events");
const mailProc = se.processes.byName("Mail");
// Locate signature popup and choose item (UI hierarchy varies)
```

**PyXA:**
```python
import PyXA

# For signature selection (UI scripting approach)
system_events = PyXA.Application("System Events")
mail_process = system_events.processes().by_name("Mail")

# Access signature popup menu
# Note: UI hierarchy varies by Mail version
compose_window = mail_process.windows()[0]  # Main compose window
signature_menu = compose_window.pop_up_buttons().by_name("Signature")
signature_menu.click()

# Select specific signature
menu_items = system_events.processes().by_name("Mail").menus()[0].menu_items()
signature_item = menu_items.by_title("Work Signature")
signature_item.click()
```

## Recursive mailbox search

**JXA:**
```javascript
function findMailbox(container, name) {
  const kids = container.mailboxes();
  for (let i = 0; i < kids.length; i++) {
    if (kids[i].name() === name) return kids[i];
  }
  for (let i = 0; i < kids.length; i++) {
    const found = findMailbox(kids[i], name);
    if (found) return found;
  }
  return null;
}
```

**PyXA:**
```python
import PyXA

def find_mailbox(container, name):
    """Recursively search for a mailbox by name"""
    # Check direct children first
    mailboxes = container.mailboxes()
    for mailbox in mailboxes:
        if mailbox.name == name:
            return mailbox

    # Recursively search child mailboxes
    for mailbox in mailboxes:
        found = find_mailbox(mailbox, name)
        if found:
            return found

    return None

# Usage
mail = PyXA.Application("Mail")
account = mail.accounts()[0]  # First account
target_mailbox = find_mailbox(account, "Archive/Sent Items")
if target_mailbox:
    print(f"Found mailbox: {target_mailbox.name}")
```

