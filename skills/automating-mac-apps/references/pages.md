# JXA Pages recipes

## Export current document to PDF
```javascript
const pages = Application("Pages");
const doc = pages.documents[0];
if (!doc) throw new Error("No document open");
doc.export({ to: Path("/tmp/export.pdf"), as: "PDF" });
JSON.stringify({ success: true });
```
