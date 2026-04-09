# PowerPoint export video workflow

## Suggested sequence
1) Ensure deck is saved.
2) Set slide timings and transitions.
3) Export with MP4 enum (39).
4) Wait for completion (export can be long).

## Example (MP4 export)
```javascript
const PpSaveAs = { MP4: 39 };
const deck = ppt.activePresentation;
const out = "/Users/you/Desktop/deck.mp4";

deck.save({ in: out, as: PpSaveAs.MP4 });
```

## Timing hint
- For long exports, wrap in a higher Apple Event timeout when running via AppleScript/JXA.

