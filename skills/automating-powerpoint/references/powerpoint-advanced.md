# PowerPoint JXA advanced patterns

## Save/export enums
```javascript
const PpSaveAs = { PPTX: 24, PDF: 32, PNG: 18, JPG: 17 };
function saveAsPDF(deck, path) {
  deck.save({ in: path, as: PpSaveAs.PDF });
}
```

## Charting via Excel interop
- Build chart in Excel, copy to clipboard, paste into PowerPoint.

```javascript
const excel = Application("Microsoft Excel");
// ... create chart in Excel, then copy
// excelChart.copy();
// ppt.activate(); slide.paste();
```

