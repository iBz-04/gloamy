---
name: pptx
description: Handle presentation decks with deliberate layout, reusable structure, and quality checks that catch visual regressions.
---

# PPTX Skill
Handle presentation decks with deliberate layout, reusable structure, and quality checks that catch visual regressions.

## When to Use

- Use this skill when working with an **existing** `.pptx` file (editing, inspecting, or exporting it).
- **On macOS**: if the user asks to "make a PowerPoint", "create a presentation", "open PowerPoint and make slides", or similar, use the `automating-powerpoint` skill instead — it creates presentations natively via JXA/osascript.
- Use this skill for template-based edits, slide rewrites, speaker note updates, and deck cleanup on existing files only.

## Workflow

1. Review the existing deck or template before editing. Note layouts, typography, color usage, chart styles, and recurring slide patterns.
2. Keep one clear visual system across the deck instead of mixing unrelated layouts.
3. Build slides with enough whitespace and strong hierarchy so titles, key numbers, and captions are easy to scan.
4. Export or render the deck after changes and inspect slide images or a PDF version before finishing.
5. Re-check slides touched by late-stage fixes because alignment and overflow issues often appear after content changes.

## Guardrails

- **On macOS, never use `python-pptx` or `pip install` to create a new PowerPoint from scratch.** Use the `automating-powerpoint` skill for creation tasks.
- Do not flatten a branded template into generic title-and-bullets slides unless the user asks.
- Avoid dense paragraphs, tiny labels, weak contrast, and repeated filler layouts.
- Treat overlap, clipped text, misaligned shapes, and inconsistent margins as bugs, not cosmetic issues.
- Preserve speaker notes, citations, and appendix content when present.
- If the environment lacks presentation tooling, explain the missing dependency and stop.

## Common Tooling

- On macOS: JXA via `osascript -l JavaScript` (see `automating-powerpoint` skill) for creating presentations.
- `libreoffice` for PDF export and final visual checks on existing files.
- Image or PDF inspection tools to verify each changed slide after rendering.
