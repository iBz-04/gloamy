# PowerPoint JXA recipes

## Add a slide with layout
```javascript
const master = deck.slideMasters[0];
const layout = master.customLayouts.byName("Title Slide");
const slide = ppt.make({ new: "slide", at: deck.slides.end, withProperties: { customLayout: layout } });
```

## Add a rectangle with text
```javascript
const shape = ppt.make({
  new: "shape",
  at: slide.shapes.end,
  withProperties: { autoShapeType: 1, left: 100, top: 100, width: 400, height: 200 }
});
shape.textFrame.textRange.content = "Automated Deck";
```

## Set transition
```javascript
const t = slide.slideShowTransition;
t.entryEffect = 12; // fade
```

