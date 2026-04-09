# PowerPoint chart-aware deck generator (pattern)

## Heuristic
- If data has 1 categorical column + 1 numeric column: pie or bar.
- If data has a time column + 1+ numeric columns: line.
- If 2 numeric columns without time: scatter.

## Skeleton workflow
```javascript
// 1) Load data (array of rows)
// 2) Detect chart type
// 3) Build chart in Excel, copy, paste into PPT

function detectChartType(rows) {
  // rows: [ [header1, header2, ...], ... ]
  const cols = rows[0].length;
  if (cols === 2) return "pie"; // assume category + value
  if (cols >= 3) return "line"; // assume time + series
  return "bar";
}

const chartType = detectChartType(data);
// Build chart in Excel accordingly, then paste to PowerPoint slide
```

Notes:
- Use Excel interop for actual chart rendering.
- Keep a template workbook with chart sheets for each type.

