# Numbers sorting patterns

## Recommended approach
- Read table data in one batch.
- Sort using JS.
- Write back via clipboard shim.

```javascript
const data = table.rows.value();
const header = data[0];
const body = data.slice(1);
body.sort((a, b) => (a[1] || 0) - (b[1] || 0));
const sorted = [header].concat(body);
// Use clipboard shim to paste sorted data
```

