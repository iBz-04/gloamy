# Messages JXA control plane

## Quick start (1:1 send)
```javascript
const Messages = Application('Messages');
Messages.includeStandardAdditions = true;

const svc = Messages.services.whose({ serviceType: 'iMessage' })[0]; // or 'SMS'
const buddy = svc.buddies.whose({ handle: '+15551234567' })[0]; // prefer handle over name

try {
  Messages.send('Message content', { to: buddy }); // app-level send routes via buddy's service
} catch (err) {
  console.log('Send failed', err);
}
```
- JXA objects are *specifiers* that trigger Apple Events; every property access is a round trip.
- `handle` (phone/email) is most reliable; `name` is contact-derived and mutable.
- Enable Standard Additions when you need clipboard, alerts, paths, or shell calls.

## Service → Buddy resolution
- Multi-protocol app: always pick a `service` first (`serviceType` is usually `iMessage` or `SMS`).
- Prefer `whose` filters for deterministic targeting:
  ```javascript
  const svc = Messages.services.whose({ serviceType: 'iMessage' })[0];
  const buddy = svc.buddies.whose({ handle: '+15550001234' })[0];
  ```
- A `Buddy` can exist for each handle per service; avoid ambiguous names.

## Reliable send pattern (JXA bridge workaround)
- The Messages JXA bridge misroutes `send`; the stable path is **app-level `send` + Buddy object**.
- Never pass a string recipient; it triggers coercion errors (`-1700`).
- Minimal helper:
  ```javascript
  function sendMessage(text, handle, svcType = 'iMessage') {
    const svc = Messages.services.whose({ serviceType: svcType })[0];
    if (!svc) throw new Error(`Service ${svcType} not found`);
    const buddy = svc.buddies.whose({ handle })[0];
    if (!buddy) throw new Error(`Buddy ${handle} not found on ${svcType}`);
    Messages.send(text, { to: buddy });
  }
  ```
- If delivery still fails, switch to UI scripting flow (see `ui-scripting-attachments.md`).

## Chats and groups
- `chat.id` maps to `chat.db` `guid` (e.g., `iMessage;+;+1555...` for 1:1, `iMessage;+;chatXXXX` for groups).
- Reading `chat.messages()` is slow for long histories; prefer SQL access (see `database-forensics.md`).
- Group creation via `send` to arrays is flaky; use an existing chat by GUID or fall back to UI scripting for multi-recipient sends.

## Window management
- `Messages.activate()` brings the app forward (use before UI automation).
- Target windows via `Messages.windows.whose({ name: 'John Doe' })` when selecting chats visually.
- Minimize background runs: `win.minimized = true` when automation should stay unobtrusive.

## Debugging checklist
- Grant Automation + Accessibility permissions; grant Full Disk Access if shelling into `chat.db`.
- Add short delays after `activate()` when switching to UI scripting.
- Wrap sends in `try/catch` and log errors; the app can drop Apple Event routing mid-run.
