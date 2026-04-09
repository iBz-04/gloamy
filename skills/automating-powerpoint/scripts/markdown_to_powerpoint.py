#!/usr/bin/env python3
"""
Markdown to PowerPoint Script - PyXA Implementation
Converts a markdown file to a PowerPoint presentation

Usage: python markdown_to_powerpoint.py input.md "Presentation Title"
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

def create_powerpoint_from_markdown(markdown_file, presentation_title):
    """Create a PowerPoint presentation from markdown file"""
    try:
        # Read markdown file
        with open(markdown_file, 'r', encoding='utf-8') as f:
            markdown_content = f.read()

        # Parse into slides
        slides = parse_markdown_slides(markdown_content)

        if not slides:
            print("No slides found in markdown file")
            return False

        # Create PowerPoint presentation
        powerpoint = PyXA.Application("Microsoft PowerPoint")

        # Create new presentation
        presentation = powerpoint.presentations().push({
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
                title_shapes = slide.shapes().filter(
                    lambda s: "title" in str(s.name()).lower()
                )
                if title_shapes:
                    title_shapes[0].text_frame().text_range().text = slide_data["title"]

            # Set content
            content_text = "\n".join(slide_data["content"])
            if content_text:
                # Find content placeholder
                content_shapes = slide.shapes().filter(
                    lambda s: "content" in str(s.name()).lower() or
                             "body" in str(s.name()).lower()
                )
                if content_shapes:
                    content_shapes[0].text_frame().text_range().text = content_text

        # Save presentation
        presentation.save()

        print(f"Created PowerPoint presentation '{presentation_title}' with {len(slides)} slides")
        print(f"Source: {markdown_file}")

        return True

    except Exception as e:
        print(f"Error creating PowerPoint presentation: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python markdown_to_powerpoint.py input.md 'Presentation Title'")
        sys.exit(1)

    markdown_file = sys.argv[1]
    title = sys.argv[2]

    success = create_powerpoint_from_markdown(markdown_file, title)
    sys.exit(0 if success else 1)