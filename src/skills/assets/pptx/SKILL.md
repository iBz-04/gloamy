---
name: pptx
description: Handle presentation decks with deliberate layout, reusable structure, and quality checks that catch visual regressions.
---

# PPTX Skill
Handle presentation decks with deliberate layout, reusable structure, and quality checks that catch visual regressions.

## When to Use

- Use this skill when the main input or output is a `.pptx` deck.
- Use it for new presentations, template-based edits, slide rewrites, speaker note updates, and deck cleanup.
- Prefer it whenever the user asks for slides, a pitch deck, a presentation, or an exported presentation file.

## Workflow

1. Review the existing deck or template before editing. Note layouts, typography, color usage, chart styles, and recurring slide patterns.
2. Keep one clear visual system across the deck instead of mixing unrelated layouts.
3. Build slides with enough whitespace and strong hierarchy so titles, key numbers, and captions are easy to scan.
4. Export or render the deck after changes and inspect slide images or a PDF version before finishing.
5. Re-check slides touched by late-stage fixes because alignment and overflow issues often appear after content changes.

## Guardrails

- Do not flatten a branded template into generic title-and-bullets slides unless the user asks.
- Avoid dense paragraphs, tiny labels, weak contrast, and repeated filler layouts.
- Treat overlap, clipped text, misaligned shapes, and inconsistent margins as bugs, not cosmetic issues.
- Preserve speaker notes, citations, and appendix content when present.
- If the environment lacks presentation tooling, explain the missing dependency and stop.

## Common Tooling

- `python-pptx` or similar libraries for deterministic slide editing.
- `libreoffice` for PDF export and final visual checks.
- Image or PDF inspection tools to verify each changed slide after rendering.
