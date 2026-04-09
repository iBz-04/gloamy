# PowerPoint export to video (notes)

## Export formats
- MP4 export uses a file format enum (often 39).

## Example (MP4)
```javascript
const PpSaveAs = { MP4: 39 };
const deck = ppt.activePresentation;
deck.save({ in: "/Users/you/Desktop/deck.mp4", as: PpSaveAs.MP4 });
```

Notes:
- Export is slow; add generous timeouts if running via Apple Events.
- Some versions require the UI frontmost to complete export.

