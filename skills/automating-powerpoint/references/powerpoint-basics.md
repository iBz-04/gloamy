# PowerPoint JXA basics

## Bootstrapping
```javascript
const ppt = Application("Microsoft PowerPoint");
ppt.includeStandardAdditions = true;
ppt.activate();
```

## Open and save
```javascript
const deck = ppt.open(Path("/Users/you/Decks/Q3.pptx"));
deck.save();
```

## New presentation
```javascript
const deck = ppt.make({ new: "presentation" });
```

