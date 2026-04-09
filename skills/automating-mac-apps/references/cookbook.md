# JXA Developer Cookbook (condensed)

## Scope
For production macOS automation using JXA (JavaScript for Automation). Use this after AppleScript discovery, when you need maintainable logic, JSON output, and pipeline integration.

## When to use JXA
- Automation logic that benefits from real JS (maps, filters, JSON).
- Pipelines where stdout must be structured JSON.
- Long-lived scripts with retries and defensive checks.

## When not to use JXA
- Cross-platform automation without Apple Events.
- UI-heavy workflows that are better in Shortcuts.
- Headless CI where GUI apps are unavailable.

## Runtime notes (practical)
- JXA runs under JavaScriptCore (not Node.js).
- Prefer synchronous patterns; async/await is unreliable.
- Use `JSON.stringify()` for stdout; `console.log` writes to stderr.

## Core mental model
- `Application("App")` is an Apple Events bridge to the app.
- Collections are often functions: `app.documents()`.
- Properties are methods for reads: `doc.name()`; setters are direct: `doc.name = "New"`.

## Helper library
Use the shared helper functions from `automating_mac_apps/references/recipes.md` (waitUntil, jsonOut, shQuote, safeGet). Add backoff and activation helpers when needed.

## Recipes index (production-focused)
- Mail: create/send, export selection, archive by date.
- Finder: list metadata, batch rename, copy by type, cleanup old files.
- Pages: template replace, export PDF, batch export, search/replace.
- Safari/Chrome: list tabs, open URLs, close by pattern.
- Keynote: create presentation, export PDF.
- Calendar/Notes: list events, create events, create notes, export notes.
- UI scripting: System Events for non-scriptable apps.
- Pipelines: JXA -> Python/Node JSON.
- CI: dry-run patterns, headless constraints.
- Debugging: structured logging, introspection.

## Recipe: JSON pipeline output (canonical)
```javascript
function run() {
  try {
    const finder = Application("Finder");
    const files = finder.desktop.files().map(f => ({ name: f.name() }));
    return { success: true, files };
  } catch (e) {
    return { success: false, error: e.message };
  }
}
JSON.stringify(run());
```

## Recipe: retry with backoff (JXA)
```javascript
function retryWithBackoff(fn, maxRetries = 3, initialDelayMs = 100) {
  let lastError;
  for (let i = 0; i < maxRetries; i++) {
    try { return fn(); } catch (e) {
      lastError = e;
      delay((initialDelayMs * Math.pow(2, i)) / 1000);
    }
  }
  throw lastError;
}
```

## CI/headless guidance
- Avoid UI scripting; prefer dictionaries or shell commands.
- Use dry-run patterns; emit JSON reports for CI parsing.
- Many GUI apps require an interactive session; fail gracefully.

