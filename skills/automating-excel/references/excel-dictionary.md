# Excel dictionary translation table

## Common mappings
```
AppleScript                         JXA
----------------------------------- ------------------------------------------
workbook "Book1"                   Excel.workbooks.byName("Book1")
worksheet "Sheet1"                 wb.worksheets.byName("Sheet1")
value of range "A1"                ws.ranges["A1"].value()
set value of range "A1" to 1        ws.ranges["A1"].value = 1
used range                          ws.usedRange
active sheet                        Excel.activeSheet
screen updating                     Excel.screenUpdating
display alerts                      Excel.displayAlerts
```

## Element vs property
- Elements -> collections (worksheets, workbooks, rows, columns).
- Properties -> camelCase attributes (screenUpdating, displayAlerts).

