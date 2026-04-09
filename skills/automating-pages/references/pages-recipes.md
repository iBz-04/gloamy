# Pages JXA recipes

## Export PDF
```javascript
const doc = pages.documents[0];
pages.export(doc, { to: Path("/Users/you/Desktop/Export.pdf"), as: "PDF" });
```

## Replace placeholders (simple)
```javascript
const doc = pages.documents[0];
const text = doc.bodyText();
const updated = text.replace(/\{\{CLIENT\}\}/g, "Acme");
doc.bodyText = updated;
```

## Paragraph styling by match
```javascript
const paras = doc.bodyText.paragraphs;
const texts = paras.objectText();
texts.forEach((t, i) => {
  if (t.includes("CONFIDENTIAL")) {
    const p = paras[i];
    p.color = [65535, 0, 0];
    p.size = 14;
  }
});
```

