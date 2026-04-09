# Numbers formulas

## Set a formula
```javascript
const cell = table.cells["B2"];
cell.value = "=SUM(A1:A10)";
```

## Freeze a formula (value only)
```javascript
const val = cell.value();
cell.value = val;
```

## Locale note
- Formula separators may be `,` or `;` depending on system locale.

