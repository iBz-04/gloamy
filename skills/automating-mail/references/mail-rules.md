# Mail rule scripts (performMailActionWithMessages)

## Handler skeleton
```javascript
function performMailActionWithMessages(messages, rule) {
  const Mail = Application("Mail");
  for (let i = 0; i < messages.length; i++) {
    const msg = messages[i];
    const subject = msg.subject();
    if (subject.includes("Urgent")) {
      msg.flaggedStatus = true;
    }
  }
}
```

Notes:
- Rule scripts must live in `~/Library/Application Scripts/com.apple.mail`.
- Keep work fast; long tasks will freeze Mail.

## Using transcripts for follow-ups (Voice Memos)
- Meeting workflows can pass transcript text into the follow-up drafting step.
- Pattern: store transcript text in a temp file or variable, parse for agenda/decisions/action items, then:
  - Inject bullets into the email body.
  - Emit follow-up reminders (delegate to `automating-reminders`).
- Keep parsing lightweight inside Mail rule scripts; offload heavy parsing to an external helper invoked via `doShellScript` if needed.
