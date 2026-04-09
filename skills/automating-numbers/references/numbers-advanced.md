# Numbers JXA advanced patterns

## Clipboard shim for bulk writes
```javascript
function bulkWrite(table, dataMatrix) {
  const app = Application.currentApplication();
  app.includeStandardAdditions = true;
  const tsv = dataMatrix.map(r => r.join("\t")).join("\n");
  app.setTheClipboardTo(tsv);
  Numbers.activate();
  delay(0.2);
  const se = Application("System Events");
  se.keystroke("v", { using: "command down" });
}
```

## 16-bit color conversion
```javascript
function toJXAColor(r, g, b) {
  return [r * 257, g * 257, b * 257];
}
```

## ObjC bridge (file existence)
```javascript
ObjC.import("Foundation");
const fm = $.NSFileManager.defaultManager;
const exists = fm.fileExistsAtPath("/Users/you/Documents/report.numbers");
```

