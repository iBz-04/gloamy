# Pages dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
front document                      pages.documents[0]
body text                           doc.bodyText
text of body text                   doc.bodyText()
paragraphs of body text             doc.bodyText.paragraphs
export front document as PDF        pages.export(doc, { to: Path("..."), as: "PDF" })
make new document with template     pages.Document({ documentTemplate: pages.templates["Blank"] }).make()
```

Notes:
- Properties become camelCase.
- Methods use `()` for reads, assignment for writes.

