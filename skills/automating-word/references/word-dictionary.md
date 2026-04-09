# Word dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
front document                      word.documents[0]
content of document                 doc.content
text of content                     doc.content.content()
make new document                   word.make({ new: 'document' })
find text                           doc.content.find
save as PDF                         doc.saveAs({ fileName: "...", fileFormat: 17 })
```

Notes:
- Use integer enums for file formats.
- Prefer Range over Selection.

