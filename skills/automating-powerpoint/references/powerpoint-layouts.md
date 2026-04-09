# PowerPoint layout presets (notes)

## Common layout names
- "Title Slide"
- "Title and Content"
- "Section Header"
- "Blank"

## Example usage
```javascript
const master = deck.slideMasters[0];
const layout = master.customLayouts.byName("Title and Content");
const slide = ppt.make({ new: "slide", at: deck.slides.end, withProperties: { customLayout: layout } });
```

Notes:
- Layout names are theme-dependent; validate via Script Editor.

