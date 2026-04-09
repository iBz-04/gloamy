# PowerPoint charting notes

## Recommended approach
- Build charts in Excel, copy to clipboard, paste into PowerPoint.
- Avoid direct chart creation in PowerPoint JXA where possible.

## Example interop flow (conceptual)
```javascript
const excel = Application("Microsoft Excel");
const ppt = Application("Microsoft PowerPoint");
// build chart in Excel
// chart.copy();
ppt.activate();
const slide = ppt.activePresentation.slides[0];
slide.paste();
```

