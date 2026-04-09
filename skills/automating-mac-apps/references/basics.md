# JXA basics (JavaScript binding for Apple Events)

## Contents
- Mental model
- Core patterns
- Standard additions
- Shell interop
- Error handling
- Runtime notes
- Runtime quirks (practical)
- ObjC bridge (when you need it)
- JXA UI scripting (System Events)
- Dictionary discovery cheat sheet (AppleScript -> JXA)

## Mental model
- Same Apple Events layer as AppleScript.
- `Application("App")` maps to the app dictionary.
- Best for JSON, data transforms, and maintainable logic.

## Core patterns
```javascript
const safari = Application("Safari");
const url = safari.windows[0].currentTab().url();
url;
```

```javascript
const finder = Application("Finder");
const names = finder.desktop.files().map(f => f.name());
JSON.stringify(names);
```

## Standard additions
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
app.displayDialog("Hello");
```

## Shell interop
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const out = app.doShellScript("whoami");
out;
```

## Error handling
```javascript
try {
  const mail = Application("Mail");
  const subjects = mail.selection().map(m => m.subject());
  JSON.stringify(subjects);
} catch (err) {
  throw new Error("Mail automation failed: " + err);
}
```

## Runtime notes
- JXA runs under JavaScriptCore (not Node.js).
- Avoid modern JS features that may not be supported.
- Prefer JSON output for pipelines.

## Runtime quirks (practical)
- `console.log` writes to stderr in `osascript`; return values go to stdout.
- Collections from apps are often special proxy objects; convert with `.map` when possible.
- `delay(seconds)` is a global provided by the automation environment.
- Some apps require `app.activate()` before accessing windows or UI state.

## ObjC bridge (when you need it)
Use ObjC to access Cocoa APIs for file IO, clipboard, or dates.
```javascript
ObjC.import("Foundation");

const str = $.NSString.alloc.initWithUTF8String("Hello");
const path = $.NSString.stringWithString("~/Desktop/jxa.txt")
  .stringByExpandingTildeInPath;
str.writeToFileAtomicallyEncodingError(path, true, $.NSUTF8StringEncoding, null);
```

## JXA UI scripting (System Events)
```javascript
const se = Application("System Events");
const safari = se.processes.byName("Safari");
Application("Safari").activate();
delay(0.2);
safari.menuBars[0].menuBarItems.byName("File").menus[0].menuItems.byName("New Tab").click();
```

## Dictionary discovery cheat sheet (AppleScript -> JXA)
Use Script Editor to learn terms, then translate with these patterns.

```text
tell application "App"         -> const app = Application("App");
current tab of front window    -> app.windows[0].currentTab()
name of every file of desktop  -> app.desktop.files().map(f => f.name())
every document                 -> app.documents()
make new document              -> app.Document({}).make()
set name of x to "Y"           -> x.name = "Y"
```

Notes:
- Collections are often functions in JXA (`documents()`), not properties.
- Properties are accessed as methods (`doc.name()`), but can be set directly (`doc.name = "New"`).
