# JXA Finder recipes

## List desktop files
```javascript
const finder = Application("Finder");
const files = finder.desktop.files().map(f => ({
  name: f.name(),
  kind: f.kind(),
  size: f.physicalSize(),
}));
JSON.stringify({ count: files.length, files });
```

## Batch rename with prefix
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const targetDir = "/tmp";
const oldPrefix = "old_";
const newPrefix = "new_";
const cmd = "cd " + shQuote(targetDir) + "; ls -1 " + shQuote(oldPrefix + "*");
const out = app.doShellScript(cmd).split("\n").filter(Boolean);
out.forEach(name => app.doShellScript("mv " + shQuote(name) + " " + shQuote(name.replace(oldPrefix, newPrefix))));
JSON.stringify({ renamed: out.length });
```
