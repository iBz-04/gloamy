---
name: docx
description: Handle `.docx` files with template-safe edits, extraction, review, and generation workflows.
---

# DOCX Skill
Handle `.docx` files with template-safe edits, extraction, review, and generation workflows.

## When to Use

- Use this skill when the main input or output is a Word-compatible document.
- Use it for document drafting, structured updates, clause edits, tracked-review style work, and text extraction from existing `.docx` files.
- If the source file is legacy `.doc`, convert it to `.docx` before making edits.

## Workflow

1. Read the document structure before editing. Check headings, tables, lists, comments, images, and page breaks.
2. Preserve the document's existing styles unless the user explicitly asks for a redesign.
3. Prefer small, targeted edits over rebuilding the full file.
4. When generating a new document, keep the output readable in Word, LibreOffice, and common previewers.
5. Write to a new output file unless the user explicitly asks you to overwrite the original.

## Guardrails

- Preserve template formatting, section order, and references.
- Keep tracked-review style requests explicit. If the user asks for visible edits or review markup, do not silently flatten those changes.
- Re-open or export the output for a final verification pass. Check for broken tables, missing images, unexpected font changes, and page layout regressions.
- If required tools are missing, report the missing dependency and stop instead of guessing.

## Common Tooling

- `pandoc` for text extraction and format conversion.
- `python-docx` or `docx`-style generators for structured document creation.
- `libreoffice` or another document renderer for final validation and PDF export.
