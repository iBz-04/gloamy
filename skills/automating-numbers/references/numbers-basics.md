# Numbers JXA basics

## Bootstrapping
```javascript
'use strict';
const Numbers = Application("Numbers");
Numbers.includeStandardAdditions = true;
Numbers.activate();
```

## Open document
```javascript
const doc = Numbers.open(Path("/Users/you/Documents/report.numbers"));
```

## Sheets and tables
```javascript
const sheet = doc.sheets.byName("Sheet 1");
const table = sheet.tables.byName("Table 1");
```

