# Pages export options matrix

## Common export targets
- PDF
- Microsoft Word (.docx)
- EPUB

## Example (PDF)
```javascript
pages.export(doc, {
  to: Path("/Users/you/Desktop/Export.pdf"),
  as: "PDF",
  withProperties: {
    imageQuality: "Best",
    includeComments: false
  }
});
```

## Example (Word)
```javascript
pages.export(doc, {
  to: Path("/Users/you/Desktop/Export.docx"),
  as: "Microsoft Word"
});
```

## Example (EPUB)
```javascript
pages.export(doc, {
  to: Path("/Users/you/Desktop/Export.epub"),
  as: "EPUB"
});
```

