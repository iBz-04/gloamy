# JXA Keynote recipes

## Export current presentation to PDF
```javascript
const keynote = Application("Keynote");
const doc = keynote.documents[0];
if (!doc) throw new Error("No presentation open");
doc.export({ to: Path("/tmp/presentation.pdf"), as: "PDF" });
JSON.stringify({ success: true });
```
