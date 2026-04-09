# Numbers dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
front document                      Numbers.documents[0]
active sheet                        doc.activeSheet
sheet "Sheet 1"                     doc.sheets.byName("Sheet 1")
 table "Table 1"                    sheet.tables.byName("Table 1")
value of cell "A1"                  table.cells["A1"].value()
value of range "A1:C10"             table.ranges["A1:C10"].value()
selection range                     table.selectionRange
```

Notes:
- Collections are specifiers; call methods to read values.
- Prefer byName access for sheets/tables.

