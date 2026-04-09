# UI scripting + attachments

Use when JXA `send` fails or you must deliver attachments (Messages has no JXA attachment API).

## Preconditions
- System Settings → Privacy & Security → Accessibility: allow the editor/terminal.
- Grant Automation permissions to send events to Messages and System Events.
- Best-effort clipboard requires Standard Additions; ObjC clipboard improves reliability.

## Copy–Paste–Send flow
1) **Prep**: `Messages.activate(); delay(0.5);` (give the window manager time).
2) **Select chat**: ensure the right conversation is frontmost (select via JXA or UI).
3) **Load clipboard**:
   - Basic: `Messages.theClipboard = Path('/path/to/file');`
   - Robust (ObjC pasteboard):
     ```javascript
     ObjC.import('AppKit');
     const pb = $.NSPasteboard.generalPasteboard;
     pb.clearContents();
     pb.writeObjects([$.NSURL.fileURLWithPath('/path/to/file')]);
     ```
4) **Inject**:
   ```javascript
   const SE = Application('System Events');
   const proc = SE.processes['Messages'];
   SE.keystroke('v', { using: 'command down' }); // paste file or rich content
   delay(0.2);
   SE.keyCode(36); // Enter to send
   ```
5) **Cleanup**: optional restore of the previously active app.

## Robustness patterns
- Replace fixed delays with wait loops (e.g., while !proc.windows.length()) when feasible.
- Guard every UI call with `try/catch` and log to a temp file for forensics.
- Keep clipboard scoped: store and restore prior contents if user context matters.
- If paste silently fails, verify Accessibility permission and that the chat input is focused.
