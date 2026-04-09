#!/usr/bin/env osascript -l JavaScript
/**
 * insert_images.js
 * Inserts images into an Apple Pages document using ObjC pasteboard bridging.
 *
 * IMPORTANT: Unlike Keynote, Pages does NOT support direct image insertion via
 * Pages.Image({file: Path(...)}). This script uses the ObjC pasteboard method
 * which is the only reliable way to insert images programmatically into Pages.
 *
 * Usage:
 *   osascript -l JavaScript insert_images.js
 *
 * Or make executable and run:
 *   chmod +x insert_images.js
 *   ./insert_images.js
 *
 * Modify the IMAGES array and DOC_PATH constants for your use case.
 */

'use strict';

// Import AppKit for NSImage and NSPasteboard
ObjC.import('AppKit');

// ============ CONFIGURATION ============
// Modify these paths for your use case

const DOC_PATH = "/path/to/your/document.pages";

const IMAGES = [
    "/path/to/image1.png",
    "/path/to/image2.png",
    "/path/to/image3.png"
];

// ============ MAIN SCRIPT ============

const Pages = Application('Pages');
Pages.includeStandardAdditions = true;

/**
 * Paste an image from a file path into the current cursor position in Pages.
 * Uses ObjC bridging to load image via NSImage and paste via NSPasteboard.
 *
 * @param {string} imagePath - Absolute path to the image file (PNG, JPEG, TIFF, etc.)
 * @returns {string} Status message
 */
function pasteImageFromFile(imagePath) {
    // Load image using NSImage
    const nsImage = $.NSImage.alloc.initWithContentsOfFile(imagePath);
    if (!nsImage) {
        return "ERROR: Failed to load image: " + imagePath;
    }

    // Get the general pasteboard and clear it
    const pasteboard = $.NSPasteboard.generalPasteboard;
    pasteboard.clearContents;

    // Set TIFF representation of the image to pasteboard
    // TIFF is used because it's a universal format that Pages accepts
    pasteboard.setDataForType(nsImage.TIFFRepresentation, $.NSPasteboardTypeTIFF);

    // Ensure Pages is frontmost
    Pages.activate();
    delay(0.3);

    // Use System Events to simulate Cmd+V paste
    const systemEvents = Application('System Events');
    systemEvents.keystroke('v', { using: 'command down' });
    delay(1); // Wait for image to be inserted

    // Add line breaks after image for spacing
    systemEvents.keystroke('\r');
    systemEvents.keystroke('\r');
    delay(0.3);

    return "SUCCESS: Pasted image: " + imagePath;
}

/**
 * Move cursor to end of document using keyboard shortcuts.
 */
function moveCursorToEnd() {
    const systemEvents = Application('System Events');
    // Cmd+End or Cmd+Shift+End to go to end of document
    systemEvents.keystroke('e', { using: ['command down', 'shift down'] });
    delay(0.3);
    // Press right arrow to deselect and position cursor at end
    systemEvents.keyCode(124);
    delay(0.3);
    // Add some line breaks
    systemEvents.keystroke('\r');
    systemEvents.keystroke('\r');
}

/**
 * Main execution function.
 */
function run() {
    try {
        // Activate Pages
        Pages.activate();
        delay(1);

        // Open the document (or use frontmost document)
        let doc;
        if (DOC_PATH !== "/path/to/your/document.pages") {
            doc = Pages.open(Path(DOC_PATH));
        } else {
            // Use frontmost document if no path specified
            doc = Pages.documents[0];
        }
        delay(1);

        // Move cursor to end of document
        moveCursorToEnd();

        // Insert each image
        const results = [];
        for (const imagePath of IMAGES) {
            if (imagePath !== "/path/to/image1.png") { // Skip placeholder paths
                const result = pasteImageFromFile(imagePath);
                results.push(result);
                console.log(result);
            }
        }

        // Save the document
        doc.save();

        return "Completed. Results:\n" + results.join('\n');

    } catch (error) {
        return "ERROR: " + error.message;
    }
}

// Execute
run();
