import sys
from PIL import Image, ImageDraw, ImageOps

def create_squircle_mask(size, radius):
    mask = Image.new("L", size, 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, size[0], size[1]), radius=radius, fill=255)
    return mask

def generate_mac_icon(input_path, output_path):
    # macOS icon guidelines for 1024x1024:
    # Inner shape is 824x824, centered (margin 100px)
    # Corner radius is ~185px (approx 22.5% of 824)
    canvas_size = (1024, 1024)
    inner_size = (824, 824)
    radius = 185

    # Load original image
    try:
        img = Image.open(input_path).convert("RGBA")
    except Exception as e:
        print(f"Failed to load {input_path}: {e}")
        return

    # Resize to the inner squircle size
    img = img.resize(inner_size, Image.Resampling.LANCZOS)

    # Create squircle mask
    mask = create_squircle_mask(inner_size, radius)

    # Apply mask
    squircle_img = Image.new("RGBA", inner_size, (0, 0, 0, 0))
    squircle_img.paste(img, (0, 0), mask=mask)

    # Create final canvas
    canvas = Image.new("RGBA", canvas_size, (0, 0, 0, 0))
    
    # Calculate center position
    x = (canvas_size[0] - inner_size[0]) // 2
    y = (canvas_size[1] - inner_size[1]) // 2
    
    # Paste squircle onto canvas
    canvas.paste(squircle_img, (x, y))

    # Save
    canvas.save(output_path)
    print(f"Successfully generated macOS-compliant icon at {output_path}")

if __name__ == "__main__":
    generate_mac_icon("desktop/src-tauri/icons/icon.png", "desktop/src-tauri/icons/app-icon.png")
