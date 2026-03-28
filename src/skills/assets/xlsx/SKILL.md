---
name: xlsx
description: Handle spreadsheet creation and editing with clean formulas, stable structure, and verification-friendly outputs.
---

# XLSX Skill
Handle spreadsheet creation and editing with clean formulas, stable structure, and verification-friendly outputs.

## When to Use

- Use this skill when the user needs `.xlsx`, `.xlsm`, `.csv`, or `.tsv` data cleaned, analyzed, reshaped, or delivered as a spreadsheet.
- Use it for workbook fixes, formula updates, dashboards, and repeatable tabular exports.
- Prefer this skill when the spreadsheet file itself is the deliverable, not just an intermediate format.

## Workflow

1. Inspect workbook sheets, headers, formulas, named ranges, charts, and formatting before editing.
2. Preserve existing workbook conventions unless the user explicitly asks for a redesign.
3. Use spreadsheet formulas for values that should remain dynamic after handoff.
4. Keep raw inputs, assumptions, calculations, and outputs clearly separated when building a new workbook.
5. Save, recalculate, and inspect the result before declaring success.

## Guardrails

- Do not replace formulas with hardcoded numbers unless the user explicitly asks for static values.
- Avoid introducing broken references, hidden circular dependencies, or inconsistent fill patterns across rows and columns.
- Keep date, currency, percent, and decimal formats consistent within each sheet.
- When cleaning messy data, preserve the original data in a separate sheet or output file when possible.
- If required spreadsheet tooling is missing, say what is missing and stop.

## Common Tooling

- `pandas` for data loading, cleaning, and reshaping.
- `openpyxl` for workbook formulas, formatting, charts, and sheet-level edits.
- `libreoffice` or another spreadsheet engine for recalculation and visual checks.
