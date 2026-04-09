# PowerPoint dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
active presentation                 ppt.activePresentation
presentation "Deck"                ppt.presentations.byName("Deck")
make new slide                      ppt.make({ new: "slide", at: deck.slides.end })
save as PDF                         deck.save({ in: "...", as: 32 })
```

Notes:
- Use integer enums for file formats.
- Prefer explicit layout selection via slide masters.

