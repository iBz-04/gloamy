# PyXA Mail Module API Reference

> **New in PyXA version 0.0.4** - Control macOS Mail.app using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Mail module. For practical examples and usage patterns, see [mail-basics.md](mail-basics.md) and [mail-recipes.md](mail-recipes.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XAMailApplication](#xamailapplication)
- [XAMailAccount](#xamailaccount)
- [XAMailbox](#xamailbox)
- [XAMailMessage](#xamailmessage)
- [XAMailOutgoingMessage](#xamailoutgoingmessage)
- [XAMailRecipient Classes](#xamailrecipient-classes)
- [XAMailAttachment](#xamailattachment)
- [XAMailHeader](#xamailheader)
- [XAMailRule](#xamailrule)
- [XAMailSignature](#xamailsignature)
- [XAMailSMTPServer](#xamailsmtpserver)
- [XAMailMessageViewer](#xamailmessageviewer)
- [Account Type Classes](#account-type-classes)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XAMailApplication (XASBApplication)
│   ├── XAMailAccount
│   │   ├── XAMailIMAPAccount
│   │   ├── XAMailPOPAccount
│   │   └── XAMailICloudAccount
│   ├── XAMailbox
│   │   └── XAMailContainer
│   ├── XAMailMessage
│   ├── XAMailOutgoingMessage
│   ├── XAMailRecipient
│   │   ├── XAMailToRecipient
│   │   ├── XAMailCcRecipient
│   │   └── XAMailBccRecipient
│   ├── XAMailAttachment
│   ├── XAMailHeader
│   ├── XAMailRule
│   │   └── XAMailRuleCondition
│   ├── XAMailSignature
│   ├── XAMailSMTPServer
│   ├── XAMailMessageViewer
│   ├── XAMailDocument
│   └── XAMailWindow (XASBWindow)
```

---

## XAMailApplication

**Bases:** `XASBApplication`

Main entry point for interacting with Mail.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | The name of the application |
| `version` | `str` | The version number of Mail.app |
| `application_version` | `str` | The build number of Mail.app |
| `frontmost` | `bool` | Whether Mail is the active application |
| `background_activity_count` | `int` | Number of background activities running |
| `primary_email` | `str` | The user's primary email address |
| `selection` | `XAMailMessageList` | Currently selected messages |

#### Mailbox Properties

| Property | Type | Description |
|----------|------|-------------|
| `inbox` | `XAMailbox` | The top-level inbox |
| `drafts_mailbox` | `XAMailbox` | The top-level drafts mailbox |
| `sent_mailbox` | `XAMailbox` | The top-level sent mailbox |
| `trash_mailbox` | `XAMailbox` | The top-level trash mailbox |
| `junk_mailbox` | `XAMailbox` | The top-level junk mailbox |
| `outbox` | `XAMailbox` | The top-level outbox |

#### Composition Settings

| Property | Type | Description |
|----------|------|-------------|
| `always_bcc_myself` | `bool` | Include user in Bcc field |
| `always_cc_myself` | `bool` | Include user in Cc field |
| `default_message_format` | `Format` | Default format for new messages |
| `quote_original_message` | `bool` | Include original text in replies |
| `include_all_original_message_text` | `bool` | Quote all or selected text |
| `same_reply_format` | `bool` | Reply in same format as original |
| `expand_group_addresses` | `bool` | Expand group addresses |
| `choose_signature_when_composing` | `bool` | Allow signature choice in compose |
| `selected_signature` | `str` | Currently selected signature name |

#### Fetch Settings

| Property | Type | Description |
|----------|------|-------------|
| `fetches_automatically` | `bool` | Auto-fetch mail at interval |
| `fetch_interval` | `int` | Minutes between fetches (-1 = auto) |
| `download_html_attachments` | `bool` | Download HTML images/attachments |

#### Display Settings

| Property | Type | Description |
|----------|------|-------------|
| `message_font` | `str` | Font name for messages |
| `message_font_size` | `float` | Font size for messages |
| `message_list_font` | `str` | Font for message list |
| `message_list_font_size` | `float` | Font size for message list |
| `fixed_width_font` | `str` | Font for plain text |
| `fixed_width_font_size` | `int` | Font size for plain text |
| `use_fixed_width_font` | `bool` | Use fixed-width for plain text |
| `color_quoted_text` | `bool` | Color quoted text |
| `level_one_quoting_color` | `QuotingColor` | Color for level 1 quotes |
| `level_two_quoting_color` | `QuotingColor` | Color for level 2 quotes |
| `level_three_quoting_color` | `QuotingColor` | Color for level 3 quotes |
| `highlight_selected_conversation` | `bool` | Highlight conversation messages |
| `check_spelling_while_typing` | `bool` | Auto spell-check |
| `new_mail_sound` | `str` | Sound for new mail ("None" to disable) |
| `should_play_other_mail_sounds` | `bool` | Play other sounds |

### Methods

#### `accounts(filter=None) → XAMailAccountList`

Returns mail accounts matching the filter.

#### `imap_accounts(filter=None) → XAMailIMAPAccountList`

Returns IMAP accounts matching the filter.

#### `pop_accounts(filter=None) → XAMailPOPAccountList`

Returns POP accounts matching the filter.

#### `mailboxes(filter=None) → XAMailboxList`

Returns mailboxes matching the filter.

#### `outgoing_messages(filter=None) → XAMailOutgoingMessageList`

Returns outgoing messages matching the filter.

#### `message_viewers(filter=None) → XAMailMessageViewerList`

Returns message viewer windows matching the filter.

#### `rules(filter=None) → XAMailRuleList`

Returns mail rules matching the filter.

#### `signatures(filter=None) → XAMailSignatureList`

Returns signatures matching the filter.

#### `smtp_servers(filter=None) → XAMailSMTPServerList`

Returns SMTP servers matching the filter.

#### `check_for_new_mail(account) → XAMailApplication`

Checks for new mail in the specified account.

#### `synchronize(account) → XAMailApplication`

Synchronizes the specified account.

#### `import_mailbox(file_path) → XAMailApplication`

Imports a mailbox from the specified file path.

---

## XAMailAccount

**Bases:** `XAObject`

Represents a mail account (base class for IMAP, POP, iCloud).

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Account name |
| `account_type` | `AccountType` | Type: pop, smtp, imap, or iCloud |
| `enabled` | `bool` | Whether account is enabled |
| `user_name` | `str` | Username for connection |
| `password` | `None` | Password (write-only) |
| `full_name` | `str` | User's full name |
| `email_addresses` | `list[str]` | Associated email addresses |
| `server_name` | `str` | Host name for connection |
| `port` | `int` | Connection port |
| `uses_ssl` | `bool` | SSL enabled |
| `authentication` | `AuthenticationMethod` | Authentication scheme |
| `account_directory` | `str` | Storage directory on disk |
| `delivery_account` | `XAMailSMTPServer` | SMTP server for sending |
| `move_deleted_messages_to_trash` | `bool` | Move deleted to trash |
| `empty_trash_on_quit` | `bool` | Delete trash on quit |
| `empty_trash_frequency` | `int` | Days before trash deletion (0=on quit, -1=never) |
| `empty_junk_messages_on_quit` | `bool` | Delete junk on quit |
| `empty_junk_messages_frequency` | `int` | Days before junk deletion |

### Methods

#### `mailboxes(filter=None) → XAMailboxList`

Returns mailboxes for this account.

---

## XAMailbox

**Bases:** `XAObject`

Represents a mailbox (folder) in Mail.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Mailbox name |
| `unread_count` | `int` | Number of unread messages |
| `account` | `XAMailAccount` | Parent account |
| `container` | `XAMailbox` | Parent mailbox (if nested) |

### Methods

#### `messages(filter=None) → XAMailMessageList`

Returns messages in this mailbox.

#### `mailboxes(filter=None) → XAMailboxList`

Returns nested mailboxes.

#### `delete()`

Permanently deletes the mailbox.

---

## XAMailMessage

**Bases:** `XAObject`

Represents an email message.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `int` | Unique identifier |
| `message_id` | `int` | Unique message ID string |
| `subject` | `str` | Subject line |
| `sender` | `str` | Sender's address |
| `reply_to` | `str` | Reply-to address |
| `content` | `XAText` | Message contents |
| `source` | `str` | Raw message source |
| `all_headers` | `str` | All headers as string |
| `date_sent` | `datetime` | Date/time sent |
| `date_received` | `datetime` | Date/time received |
| `message_size` | `int` | Size in bytes |
| `mailbox` | `XAMailbox` | Containing mailbox |
| `background_color` | `HighlightColor` | Background highlight color |

#### Status Properties

| Property | Type | Description |
|----------|------|-------------|
| `read_status` | `bool` | Whether read |
| `flagged_status` | `bool` | Whether flagged |
| `flag_index` | `int` | Flag index (-1 = not flagged) |
| `deleted_status` | `bool` | Whether deleted |
| `junk_mail_status` | `bool` | Whether marked as junk |
| `was_replied_to` | `bool` | Whether replied to |
| `was_forward` | `bool` | Whether forwarded |
| `was_redirected` | `bool` | Whether redirected |

### Methods

#### `open() → XAMailMessage`

Opens the message in a separate window.

#### `delete()`

Permanently deletes the message.

#### `forward(open_window=True) → XAMailOutgoingMessage`

Creates a forward of the message.

#### `reply(open_window=True, reply_all=False) → XAMailOutgoingMessage`

Creates a reply to the message.

#### `redirect(open_window=True) → XAMailOutgoingMessage`

Creates a redirect of the message.

#### Recipient Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `to_recipients(filter=None)` | `XAMailToRecipientList` | Primary recipients |
| `cc_recpients(filter=None)` | `XAMailCcRecipientList` | CC recipients |
| `bcc_recipients(filter=None)` | `XAMailBccRecipientList` | BCC recipients |
| `recipients(filter=None)` | `XAMailRecipientList` | All recipients |
| `headers(filter=None)` | `XAMailHeaderList` | Message headers |
| `mail_attachments(filter=None)` | `XAMailAttachmentList` | Attachments |

---

## XAMailOutgoingMessage

**Bases:** `XAObject`

Represents an outgoing (draft) message.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `int` | Unique identifier |
| `subject` | `str` | Subject line |
| `sender` | `str` | Sender address |
| `content` | `XAText` | Message contents |
| `message_signature` | `XAMailSignature` | Message signature |
| `visible` | `bool` | Whether window is shown |

### Methods

#### `send() → bool`

Sends the message. Returns success status.

#### `save()`

Saves the message as a draft.

#### `delete()`

Permanently deletes the outgoing message.

#### `close(save=SaveOption.YES)`

Closes the message window.

---

## XAMailRecipient Classes

### XAMailRecipient (Base)

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Display name |
| `address` | `str` | Email address |

### XAMailToRecipient

Primary (To:) recipient. Inherits from `XAMailRecipient`.

### XAMailCcRecipient

CC recipient. Inherits from `XAMailRecipient`.

### XAMailBccRecipient

BCC recipient. Inherits from `XAMailRecipient`.

---

## XAMailAttachment

**Bases:** `XAObject`

Represents a message attachment.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Attachment filename |
| `mime_type` | `str` | MIME type (e.g., "text/plain") |
| `file_size` | `int` | Size in bytes |
| `downloaded` | `bool` | Whether downloaded |

### Methods

#### `delete()`

Permanently deletes the attachment.

---

## XAMailHeader

**Bases:** `XAObject`

Represents a message header.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Header name |
| `content` | `str` | Header value |

---

## XAMailRule

**Bases:** `XAObject`

Represents a mail filtering rule.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Rule name |
| `enabled` | `bool` | Whether enabled |
| `all_conditions_must_be_met` | `bool` | AND vs OR conditions |
| `stop_evaluating_rules` | `bool` | Stop after match |

#### Action Properties

| Property | Type | Description |
|----------|------|-------------|
| `delete_message` | `bool` | Delete matching messages |
| `mark_read` | `bool` | Mark as read |
| `mark_flagged` | `bool` | Mark as flagged |
| `mark_flag_index` | `int` | Flag index (-1 = disabled) |
| `color_message` | `HighlightColor` | Apply color |
| `highlight_text_using_color` | `bool` | Color text vs background |
| `move_message` | `XAMailbox` | Move to mailbox |
| `copy_message` | `XAMailbox` | Copy to mailbox |
| `should_move_message` | `bool` | Has move action |
| `should_copy_message` | `bool` | Has copy action |
| `forward_message` | `str` | Forward addresses (comma-separated) |
| `forward_text` | `str` | Prepend text for forward |
| `redirect_message` | `str` | Redirect addresses |
| `reply_text` | `str` | Auto-reply text |
| `run_script` | `str` | AppleScript file path |
| `play_sound` | `str` | Sound name or path |

### Methods

#### `rule_conditions(filter=None) → XAMailRuleConditionList`

Returns conditions for this rule.

#### `delete()`

Permanently deletes the rule.

---

## XAMailRuleCondition

**Bases:** `XAObject`

Represents a condition within a mail rule.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `rule_type` | `RuleType` | Type of condition |
| `qualifier` | `RuleQualifier` | Comparison qualifier |
| `expression` | `str` | Expression to match |
| `header` | `str` | Header key (for header rules) |

### Methods

#### `delete()`

Permanently deletes the rule condition.

---

## XAMailSignature

**Bases:** `XAObject`

Represents an email signature.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Signature name |
| `content` | `XAText` | Signature content |

### Methods

#### `delete()`

Permanently deletes the signature.

---

## XAMailSMTPServer

**Bases:** `XAObject`

Represents an SMTP server configuration.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Server name |
| `account_type` | `AccountType` | Account type |
| `server_name` | `str` | Host name |
| `port` | `int` | Connection port |
| `user_name` | `str` | Username |
| `password` | `None` | Password (write-only) |
| `uses_ssl` | `bool` | SSL enabled |
| `enabled` | `bool` | Whether enabled |
| `authentication` | `AuthenticationMethod` | Auth scheme |

---

## XAMailMessageViewer

**Bases:** `XAObject`

Represents the main message viewer window.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `int` | Unique identifier |
| `window` | `XAMailWindow` | The window object |
| `mailbox_list_visible` | `bool` | Mailbox list shown |
| `preview_pane_is_visible` | `bool` | Preview pane shown |
| `sort_column` | `ViewerColumn` | Sort column |
| `sort_ascending` | `bool` | Sort direction |
| `visible_columns` | `list[str]` | Visible columns |

#### Mailbox Properties

| Property | Type | Description |
|----------|------|-------------|
| `inbox` | `XAMailbox` | Top-level inbox |
| `drafts_mailbox` | `XAMailbox` | Top-level drafts |
| `sent_mailbox` | `XAMailbox` | Top-level sent |
| `trash_mailbox` | `XAMailbox` | Top-level trash |
| `junk_mailbox` | `XAMailbox` | Top-level junk |
| `outbox` | `XAMailbox` | Top-level outbox |

#### Selection Properties

| Property | Type | Description |
|----------|------|-------------|
| `selected_mailboxes` | `XAMailboxList` | Selected mailboxes |
| `selected_messages` | `XAMailMessageList` | Selected messages |
| `visible_messages` | `XAMailMessageList` | Displayed messages |

### Methods

#### `messages(filter=None) → XAMailMessageList`

Returns messages matching the filter.

---

## Account Type Classes

### XAMailIMAPAccount

**Bases:** `XAMailAccount`

IMAP-specific account properties.

| Property | Type | Description |
|----------|------|-------------|
| `message_caching` | `CachingPolicy` | Caching policy |
| `store_drafts_on_server` | `bool` | Store drafts on server |
| `store_sent_messages_on_server` | `bool` | Store sent on server |
| `store_junk_mail_on_server` | `bool` | Store junk on server |
| `store_deleted_messages_on_server` | `bool` | Store deleted on server |
| `compact_mailboxes_when_closing` | `bool` | Auto-compact on close |

### XAMailPOPAccount

**Bases:** `XAMailAccount`

POP-specific account properties.

| Property | Type | Description |
|----------|------|-------------|
| `delete_mail_on_server` | `bool` | Delete after download |
| `delete_messages_when_moved_from_inbox` | `bool` | Delete on move |
| `delayed_message_deletion_interval` | `int` | Days before server deletion |
| `big_message_warning_size` | `int` | Size threshold for warning (-1 = no warning) |

### XAMailICloudAccount

**Bases:** `XAMailAccount`

iCloud account (uses base account properties).

---

## List Classes

All list classes support fast enumeration and bulk property access.

### Common List Methods

```python
# Bulk property access
messages = mailbox.messages()
subjects = messages.subject()        # → list[str]
senders = messages.sender()          # → list[str]
dates = messages.date_received()     # → list[datetime]

# Filtering
unread = messages.by_read_status(False)
flagged = messages.by_flagged_status(True)
from_sender = messages.by_sender("user@example.com")
```

### XAMailMessageList

| Method | Returns |
|--------|---------|
| `subject()` | `list[str]` |
| `sender()` | `list[str]` |
| `content()` | `list[str]` |
| `date_sent()` | `list[datetime]` |
| `date_received()` | `list[datetime]` |
| `read_status()` | `list[bool]` |
| `flagged_status()` | `list[bool]` |
| `junk_mail_status()` | `list[bool]` |
| `message_size()` | `list[int]` |
| `mailbox()` | `XAMailboxList` |

### XAMailboxList

| Method | Returns |
|--------|---------|
| `name()` | `list[str]` |
| `unread_count()` | `list[int]` |
| `account()` | `XAMailAccountList` |
| `messages()` | Combined messages |

### XAMailAccountList

| Method | Returns |
|--------|---------|
| `name()` | `list[str]` |
| `email_addresses()` | `list[list[str]]` |
| `enabled()` | `list[bool]` |
| `server_name()` | `list[str]` |
| `mailboxes()` | Combined mailboxes |

---

## Enumerations

### AccountType

| Value | Description |
|-------|-------------|
| `IMAP` | IMAP account |
| `POP` | POP account |
| `SMTP` | SMTP server |
| `ICLOUD` | iCloud account |
| `UNKNOWN` | Unknown type |

### AuthenticationMethod

| Value | Description |
|-------|-------------|
| `PASSWORD` | Clear text password |
| `APOP` | APOP |
| `KERBEROS5` | Kerberos V5 (GSSAPI) |
| `NTLM` | NTLM |
| `MD5` | CRAM-MD5 |
| `EXTERNAL` | TLS client certificate |
| `APPLE_TOKEN` | Apple token |
| `NONE` | No authentication |

### Format

| Value | Description |
|-------|-------------|
| `PLAIN_MESSAGE` | Plain text |
| `RICH_MESSAGE` | Rich text/HTML |
| `NATIVE` | Native format |

### HighlightColor

| Value | Description |
|-------|-------------|
| `BLUE` | Blue |
| `GRAY` | Gray |
| `GREEN` | Green |
| `NONE` | No color |
| `ORANGE` | Orange |
| `OTHER` | Other color |
| `PURPLE` | Purple |
| `RED` | Red |
| `YELLOW` | Yellow |

### QuotingColor

| Value | Description |
|-------|-------------|
| `BLUE` | Blue |
| `GREEN` | Green |
| `ORANGE` | Orange |
| `OTHER` | Other |
| `PURPLE` | Purple |
| `RED` | Red |
| `YELLOW` | Yellow |

### RuleType

| Value | Description |
|-------|-------------|
| `FROM_HEADER` | From header |
| `TO_HEADER` | To header |
| `CC_HEADER` | Cc header |
| `TO_OR_CC_HEADER` | To or Cc header |
| `SUBJECT_HEADER` | Subject header |
| `HEADER_KEY` | Arbitrary header key |
| `ANY_RECIPIENT` | Any recipient |
| `MESSAGE_CONTENT` | Message content |
| `ACCOUNT` | Account |
| `ATTACHMENT_TYPE` | Attachment type |
| `MESSAGE_IS_JUNK_MAIL` | Is junk mail |
| `SENDER_IS_IN_MY_CONTACTS` | Sender in contacts |
| `SENDER_IS_NOT_IN_MY_CONTACTS` | Sender not in contacts |
| `SENDER_IS_IN_MY_PREVIOUS_RECIPIENTS` | Sender in previous recipients |
| `SENDER_IS_NOT_IN_MY_PREVIOUS_RECIPIENTS` | Sender not in previous recipients |
| `SENDER_IS_MEMBER_OF_GROUP` | Sender in group |
| `SENDER_IS_NOT_MEMBER_OF_GROUP` | Sender not in group |
| `SENDER_IS_VIP` | Sender is VIP |
| `MATCHES_EVERY_MESSAGE` | Every message |

### RuleQualifier

| Value | Description |
|-------|-------------|
| `BEGINS_WITH_VALUE` | Begins with |
| `ENDS_WITH_VALUE` | Ends with |
| `DOES_CONTAIN_VALUE` | Contains |
| `DOES_NOT_CONTAIN_VALUE` | Does not contain |
| `EQUAL_TO_VALUE` | Equals |
| `GREATER_THAN_VALUE` | Greater than |
| `LESS_THAN_VALUE` | Less than |
| `NONE` | No qualifier |

### ViewerColumn

| Value | Description |
|-------|-------------|
| `ATTACHMENTS` | Attachment count |
| `DATE_RECEIVED` | Date received |
| `DATE_SENT` | Date sent |
| `DATE_LAST_SAVED` | Draft save date |
| `FLAGS` | Message flags |
| `FROM` | Sender name |
| `MAILBOX` | Mailbox name |
| `MESSAGE_COLOR` | Sort by color |
| `MESSAGE_STATUS` | Read/replied status |
| `NUMBER` | Message number |
| `RECIPIENTS` | Recipients |
| `SIZE` | Message size |
| `SUBJECT` | Subject |

### CachingPolicy

| Value | Description |
|-------|-------------|
| `ALL_MESSAGES_AND_THEIR_ATTACHMENTS` | Cache all |
| `ALL_MESSAGES_BUT_OMIT_ATTACHMENTS` | Cache without attachments |
| `DO_NOT_KEEP_COPIES_OF_ANY_MESSAGES` | Deprecated (maps to omit attachments) |
| `ONLY_MESSAGES_I_HAVE_READ` | Deprecated (maps to omit attachments) |

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Mail app | `mail = PyXA.Application("Mail")` |
| Get inbox | `inbox = mail.inbox` |
| Get all messages | `messages = inbox.messages()` |
| Get unread messages | `messages.by_read_status(False)` |
| Get flagged messages | `messages.by_flagged_status(True)` |
| Open a message | `message.open()` |
| Create new message | `msg = mail.outgoing_messages()[0]` |
| Send message | `msg.send()` |
| Check for mail | `mail.check_for_new_mail(account)` |

### Message Composition

```python
import PyXA

mail = PyXA.Application("Mail")

# Create outgoing message
# (Use Mail's make command or reply/forward)
msg = message.reply(open_window=True)
msg.subject = "Re: " + message.subject
msg.content = "Thank you for your message..."
msg.send()
```

### Filtering Examples

```python
# Get messages from specific sender
from_john = messages.by_sender("john@example.com")

# Get unread messages
unread = messages.by_read_status(False)

# Get messages by subject
important = messages.by_subject("Important")

# Bulk access
all_subjects = messages.subject()  # Returns list[str]
all_senders = messages.sender()    # Returns list[str]
```

---

## See Also

- [PyXA Mail Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/mail.html) - Official PyXA documentation
- [mail-basics.md](mail-basics.md) - Getting started with Mail automation
- [mail-recipes.md](mail-recipes.md) - Common automation patterns
- [mail-advanced.md](mail-advanced.md) - Advanced techniques
- [mail-rules.md](mail-rules.md) - Rule automation
