# Pages template strategy

## Recommended workflow
1) Create a template document with pre-built tables, charts, and styles.
2) Save it as a .pages file in a known path.
3) Open template, duplicate it, and inject data.

## Rationale
- JXA creation of complex objects is brittle.
- Templates provide reliable layout and styles.

## Template copy pattern
```javascript
const templatePath = Path("/Users/you/Templates/Report.pages");
const doc = pages.open(templatePath);
// Modify content...
const outPath = Path("/Users/you/Desktop/Report_Final.pages");
doc.save({ in: outPath });
```

