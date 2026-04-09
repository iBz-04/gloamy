#!/usr/bin/env osascript -l JavaScript
/**
 * create_skills_spreadsheet.js
 * Creates a Numbers spreadsheet with one tab per skill,
 * listing file paths and sizes for each file in the skill.
 * THIS IS JUST AN EXAMPLE SCRIPT.
 */

'use strict';

const Numbers = Application("Numbers");
const Finder = Application("Finder");
const app = Application.currentApplication();
app.includeStandardAdditions = true;

Numbers.includeStandardAdditions = true;

const SKILLS_PATH = "/Users/richardhightower/clients/spillwave/src/skill-foundary-agent/using_apple_automation_foundary/automating-mac-apps-plugin/plugins/automating-mac-apps-plugin/skills";

// Get all files in a directory recursively
function getFilesInDir(dirPath) {
  const files = [];
  try {
    const result = app.doShellScript(`find "${dirPath}" -type f 2>/dev/null`);
    if (result) {
      // JXA shell script returns \r for newlines
      const paths = result.split(/[\r\n]+/).filter(p => p.length > 0);
      for (const filePath of paths) {
        try {
          const sizeResult = app.doShellScript(`stat -f%z "${filePath}" 2>/dev/null || echo 0`);
          const size = parseInt(sizeResult, 10) || 0;
          // Make path relative to skill directory
          const relativePath = filePath.replace(dirPath + "/", "");
          files.push({
            path: relativePath,
            size: size
          });
        } catch (e) {
          // Skip files we can't stat
        }
      }
    }
  } catch (e) {
    // Empty directory or error
  }
  return files;
}

// Format file size for display
function formatSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}

// Get list of skills
function getSkills() {
  const result = app.doShellScript(`ls "${SKILLS_PATH}"`);
  // JXA shell script returns \r for newlines
  return result.split(/[\r\n]+/).filter(s => s.length > 0);
}

function run() {
  try {
    // Get all skills
    const skills = getSkills();

    // Activate Numbers and create new document
    Numbers.activate();
    delay(0.5);

    // Create new document
    const doc = Numbers.Document();
    Numbers.documents.push(doc);
    delay(0.5);

    // Get the document we just created
    const activeDoc = Numbers.documents[0];

    // Process each skill
    let isFirstSheet = true;

    for (const skillName of skills) {
      const skillPath = SKILLS_PATH + "/" + skillName;
      const files = getFilesInDir(skillPath);

      let sheet;

      if (isFirstSheet) {
        // Use the default first sheet
        sheet = activeDoc.sheets[0];
        sheet.name = skillName;
        isFirstSheet = false;
      } else {
        // Add a new sheet
        const newSheet = Numbers.Sheet({ name: skillName });
        activeDoc.sheets.push(newSheet);
        sheet = activeDoc.sheets[activeDoc.sheets.length - 1];
      }

      delay(0.2);

      // Get the table in this sheet
      const table = sheet.tables[0];

      // Set up header row
      table.rows[0].cells[0].value = "Path";
      table.rows[0].cells[1].value = "Size";

      // Make header bold by selecting and formatting
      // (JXA Numbers doesn't have direct bold control, but we set values)

      // Add file data
      for (let i = 0; i < files.length; i++) {
        const rowIndex = i + 1; // Skip header row

        // Ensure we have enough rows
        while (table.rows.length <= rowIndex) {
          table.rows.push(Numbers.Row());
        }

        // Set cell values
        table.rows[rowIndex].cells[0].value = files[i].path;
        table.rows[rowIndex].cells[1].value = formatSize(files[i].size);
      }

      // Resize columns to fit content (approximate)
      delay(0.1);
    }

    return `Created spreadsheet with ${skills.length} sheets (tabs) for skills`;

  } catch (error) {
    return "Error: " + error.message;
  }
}
