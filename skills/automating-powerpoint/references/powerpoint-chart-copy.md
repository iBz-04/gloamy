# Excel chart -> PowerPoint paste (example)

## End-to-end flow
```javascript
const excel = Application("Microsoft Excel");
const ppt = Application("Microsoft PowerPoint");
excel.activate();

// Build chart in Excel (assumes data and chart already exist)
const wb = excel.activeWorkbook;
const sheet = wb.worksheets[0];
const chart = sheet.chartObjects[0];
chart.copy();

// Paste into PowerPoint
ppt.activate();
const deck = ppt.activePresentation;
const slide = deck.slides[0];
slide.paste();
```

Notes:
- Prefer a template workbook with a prepared chart object.
- Copy/paste is more reliable than PowerPoint chart creation.

