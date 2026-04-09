# Numbers JXA recipes

## Read table values (batch)
```javascript
const values = table.rows.value();
```

## Set a single cell
```javascript
table.rows[1].cells[1].value = "OK";
```

## Basic formatting
```javascript
const cell = table.rows[1].cells[1];
cell.backgroundColor = [65535, 0, 0];
cell.textColor = [0, 0, 0];
```

