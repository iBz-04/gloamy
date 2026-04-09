# Excel JXA pivot tables (practical guidance)

## Reality check
- Pivot table creation via JXA is brittle.
- Prefer VBA macros for creating pivots; use JXA to invoke them.

## VBA workflow
1) Record or author a VBA macro in the workbook (e.g., `BuildPivot`).
2) Call it from JXA:
```javascript
Excel.run("BuildPivot");
```

## If you must attempt JXA
- Use explicit range addresses as strings (e.g., "Sheet1!A1:D100").
- Build a pivot cache via VBA or existing templates.

