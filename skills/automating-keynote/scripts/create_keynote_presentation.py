#!/usr/bin/env python3
"""
Create Keynote Presentation Script - PyXA Implementation
Creates a new Keynote presentation with basic content

Usage: python create_keynote_presentation.py "Presentation Title" ["Theme Name"] ["Save Path"]
"""

import sys
import PyXA
import os
import subprocess

def create_keynote_presentation(title, theme_name=None, save_path=None):
    """Create a new Keynote presentation"""
    try:
        keynote = PyXA.Application("Keynote")

        # Create new presentation (make returns a PyXA object)
        presentation = keynote.documents().push(keynote.class_("document"))
        if title:
            try:
                presentation.name = title
            except Exception:
                pass

        # Set theme if specified
        if theme_name:
            try:
                # Try to find and set the theme
                themes = keynote.themes()
                target_theme = None
                for theme in themes:
                    if theme_name.lower() in theme.name().lower():
                        target_theme = theme
                        break

                if target_theme:
                    presentation.theme = target_theme
                    print(f"Applied theme: {target_theme.name()}")
                else:
                    print(f"Theme '{theme_name}' not found, using default")
            except Exception as e:
                print(f"Could not set theme: {e}")

        # Add slides (best effort; some builds may have an initial slide already)
        try:
            slides = presentation.slides()
            if len(slides) == 0:
                slides.push()
            slides.push()
            slides.push()
        except Exception as e:
            print(f"Warning: could not add slides via PyXA: {e}")

        print(f"Created Keynote presentation '{title}' with {len(presentation.slides())} slides")
        print("Slide structure:")
        for i, slide in enumerate(presentation.slides(), 1):
            print(f"  Slide {i}: {slide.slide_number()}")

        # Save the presentation
        if save_path:
            save_path = os.path.abspath(save_path)
            presentation.save({"in": save_path})
            print(f"Saved to: {save_path}")
        else:
            presentation.save()

        return True

    except Exception as e:
        print(f"PyXA path failed: {e}; attempting AppleScript fallback.")
        if not save_path:
            print("No save path provided; cannot fallback to AppleScript save.")
            return False
        save_path = os.path.abspath(save_path)
        try:
            script = f'''
            tell application "Keynote"
                set newDoc to make new document
                try
                    set the name of newDoc to "{title}"
                end try
                save newDoc in POSIX file "{save_path}"
            end tell
            '''
            subprocess.run(["osascript", "-e", script], check=True)
            print(f"Saved to: {save_path} (AppleScript fallback)")
            return True
        except Exception as e2:
            print(f"AppleScript fallback failed: {e2}")
            return False

if __name__ == "__main__":
    title = sys.argv[1] if len(sys.argv) > 1 else "Untitled"
    theme = sys.argv[2] if len(sys.argv) > 2 else None
    save_path = sys.argv[3] if len(sys.argv) > 3 else None

    success = create_keynote_presentation(title, theme, save_path)
    sys.exit(0 if success else 1)
