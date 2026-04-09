# Excel JXA charting

## Preferred approach
- Use VBA macros for complex chart creation and formatting.
- Use JXA to populate data ranges and invoke VBA.

## Minimal chart example (if supported)
```javascript
// Assumes a chart can be created from a selected range
const range = ws.ranges["A1:B10"];
range.select();
Excel.run("CreateChartFromSelection"); // VBA macro
```

## Notes
- JXA chart APIs may be incomplete or unstable across Excel versions.
- For consistent output, store a template workbook with prebuilt charts and update its data ranges.

