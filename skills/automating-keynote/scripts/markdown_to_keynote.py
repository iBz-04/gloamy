#!/usr/bin/env python3
"""
Markdown to Keynote Script - PyXA Implementation
Converts a markdown file to a Keynote presentation

Usage: python markdown_to_keynote.py input.md "Presentation Title"
"""

import sys
import PyXA
import re

def parse_markdown_slides(markdown_content):
    """Parse markdown content into slides based on headers"""
    lines = markdown_content.split('\n')
    slides = []
    current_slide = {"title": "", "content": []}

    for line in lines:
        line = line.strip()
        if not line:
            continue

        # Check for headers (slides)
        if line.startswith('# '):
            # Save previous slide if it has content
            if current_slide["title"] or current_slide["content"]:
                slides.append(current_slide)

            # Start new slide
            current_slide = {
                "title": line[2:].strip(),
                "content": []
            }
        elif line.startswith('## '):
            # Subheader - could be a new slide or content
            if current_slide["content"]:  # If we have content, this might be a new slide
                slides.append(current_slide)
                current_slide = {
                    "title": line[3:].strip(),
                    "content": []
                }
            else:
                current_slide["content"].append(f"## {line[3:].strip()}")
        elif line.startswith('- ') or line.startswith('* '):
            # List item
            current_slide["content"].append(line)
        else:
            # Regular content
            current_slide["content"].append(line)

    # Add final slide
    if current_slide["title"] or current_slide["content"]:
        slides.append(current_slide)

    return slides

def create_keynote_from_markdown(markdown_file, presentation_title):
    """Create a Keynote presentation from markdown file"""
    try:
        # Read markdown file
        with open(markdown_file, 'r', encoding='utf-8') as f:
            markdown_content = f.read()

        # Parse into slides
        slides = parse_markdown_slides(markdown_content)

        if not slides:
            print("No slides found in markdown file")
            return False

        # Create Keynote presentation
        keynote = PyXA.Application("Keynote")

        # Create new presentation
        presentation = keynote.documents().push({
            "name": presentation_title
        })

        # Add slides
        for i, slide_data in enumerate(slides):
            if i == 0:
                # First slide already exists, modify it
                slide = presentation.slides()[0]
            else:
                # Add new slide
                slide = presentation.slides().push()

            # Set title
            if slide_data["title"]:
                # Find title placeholder and set text
                title_placeholders = slide.placeholders().filter(
                    lambda p: "title" in str(p.tag()).lower()
                )
                if title_placeholders:
                    title_placeholders[0].text = slide_data["title"]

            # Set content
            content_text = "\n".join(slide_data["content"])
            if content_text:
                # Find body placeholder
                body_placeholders = slide.placeholders().filter(
                    lambda p: "body" in str(p.tag()).lower() or "content" in str(p.tag()).lower()
                )
                if body_placeholders:
                    body_placeholders[0].text = content_text

        # Save presentation
        presentation.save()

        print(f"Created Keynote presentation '{presentation_title}' with {len(slides)} slides")
        print(f"Source: {markdown_file}")

        return True

    except Exception as e:
        print(f"Error creating Keynote presentation: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python markdown_to_keynote.py input.md 'Presentation Title'")
        sys.exit(1)

    markdown_file = sys.argv[1]
    title = sys.argv[2]

    success = create_keynote_from_markdown(markdown_file, title)
    sys.exit(0 if success else 1)