# Keynote chart-aware deck generator (pattern)

## Heuristic
- If data has 1 categorical column + 1 numeric column: pie or bar.
- If data has a time column + 1+ numeric columns: line.
- If 2 numeric columns without time: scatter.

## Skeleton workflow
```javascript
function detectChartType(rows) {
  const cols = rows[0].length;
  if (cols === 2) return "pie";
  if (cols >= 3) return "line";
  return "bar";
}

const chartType = detectChartType(data);
// Use AppleScript bridge to add chart in Keynote, or use a chart template slide.
```

Notes:
- For reliability, prefer a template Keynote with prebuilt chart slides.
- Use the chart bridge pattern from the Keynote advanced reference.

