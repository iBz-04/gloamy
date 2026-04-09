#!/usr/bin/env python3
"""
Export Keynote Presentation Script - AppleScript Implementation
Exports Keynote presentation to various formats

Note: This script currently uses AppleScript for reliable Keynote automation.
PyXA research showed rich Keynote APIs (XAKeynoteApplication, XAKeynoteDocument.export(),
ExportFormat enums) but the document opening mechanism needs further investigation.
The AppleScript approach provides consistent, working functionality.

Usage: python export_keynote_presentation.py "presentation.key" "output.pdf" ["PDF"|"PowerPoint"|"HTML"]
"""

import sys
from pathlib import Path

def export_keynote_presentation(input_file, output_file, format_type="PDF"):
    """Export Keynote presentation to specified format"""

    # Validate format type
    supported_formats = ["PDF", "POWERPOINT", "HTML"]
    if format_type.upper() not in supported_formats:
        print(f"Unsupported format: {format_type}. Supported: {', '.join(supported_formats)}")
        return False

    print("Attempting PyXA export...")

    # Try PyXA approach first (simplified - just use AppleScript for now)
    # TODO: Implement proper PyXA Keynote export once API is better understood
    return export_with_applescript(input_file, output_file, format_type)

def export_with_applescript(input_file, output_file, format_type="PDF"):
    """Export using AppleScript"""
    try:
        import subprocess

        script = f'''
        tell application "Keynote"
            set theDoc to open POSIX file "{Path(input_file).resolve()}"
            tell theDoc
                if "{format_type.upper()}" is "PDF" then
                    export to POSIX file "{Path(output_file).resolve()}" as PDF
                else if "{format_type.upper()}" is "POWERPOINT" then
                    export to POSIX file "{Path(output_file).resolve()}" as Microsoft PowerPoint
                else if "{format_type.upper()}" is "HTML" then
                    export to POSIX file "{Path(output_file).resolve()}" as HTML
                end if
            end tell
            close theDoc saving no
        end tell
        '''
        subprocess.run(["osascript", "-e", script], check=True)
        print(f"Successfully exported to: {output_file}")
        print(f"Format: {format_type}")
        return True
    except Exception as e:
        print(f"Export failed: {e}")
        return False

    try:
        script = f'''
        tell application "Keynote"
            set theDoc to open POSIX file "{Path(input_file).resolve()}"
            tell theDoc
                if "{format_type.upper()}" is "PDF" then
                    export to POSIX file "{Path(output_file).resolve()}" as PDF
                else if "{format_type.upper()}" is "POWERPOINT" then
                    export to POSIX file "{Path(output_file).resolve()}" as Microsoft PowerPoint
                else if "{format_type.upper()}" is "HTML" then
                    export to POSIX file "{Path(output_file).resolve()}" as HTML
                end if
            end tell
            close theDoc saving no
        end tell
        '''

        subprocess.run(["osascript", "-e", script], check=True)
        print(f"Successfully exported to: {output_file}")
        print(f"Format: {format_type}")
        return True

    except subprocess.CalledProcessError as e:
        print(f"AppleScript export failed: {e}")
        return False
    except Exception as e:
        print(f"Unexpected error during export: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python export_keynote_presentation.py 'input.key' 'output.pdf' ['PDF'|'PowerPoint'|'HTML']")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]
    format_type = sys.argv[3] if len(sys.argv) > 3 else "PDF"

    success = export_keynote_presentation(input_file, output_file, format_type)
    sys.exit(0 if success else 1)