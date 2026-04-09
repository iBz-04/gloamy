# PyXA Browser Integration

## Installation
```bash
pip install PyXA
```

## Browser Classes
```python
# Chrome automation
chrome = PyXA.Application("Google Chrome")

# Arc with spaces
arc = PyXA.Application("Arc")
work_space = arc.spaces().by_title("Work")
work_space.focus()
```

## Tab Management
```python
# Get all tabs
tabs = chrome.windows()[0].tabs()

# Create new tab
new_tab = chrome.new_tab("https://example.com")

# Close specific tabs
for tab in tabs:
    if "unwanted" in str(tab.url()):
        tab.close()
```

## JavaScript Execution
```python
# Execute JavaScript in current tab
content = chrome.current_tab().execute_javascript("document.body.innerText")

# Extract page metadata
meta_description = chrome.current_tab().execute_javascript("""
    const meta = document.querySelector('meta[name="description"]');
    return meta ? meta.getAttribute('content') : null;
""")
```

## Bookmark Operations
```python
# Add current page to bookmarks
current_tab = chrome.windows()[0].active_tab
chrome.bookmarks_bar.make("bookmark", {
    "title": current_tab.title(),
    "url": str(current_tab.url())
})

# Access bookmarks
bookmarks = chrome.bookmarks_bar.bookmark_items()
for bookmark in bookmarks:
    if "research" in bookmark.title().lower():
        print(f"Research: {bookmark.title()} - {bookmark.url()}")
```

## Window Management
```python
# Create new window
new_window = chrome.new_window("https://example.com")

# Minimize inactive windows
windows = chrome.windows()
for window in windows[1:]:  # Skip front window
    if not window.minimized():
        window.minimized = True
```

## Arc Browser Spaces
```python
# Arc-specific features
arc = PyXA.Application("Arc")

# List all spaces
spaces = arc.spaces()
for space in spaces:
    print(f"Space: {space.name()}")

# Switch to specific space
work_space = spaces.by_title("Work")
work_space.focus()
```

## Error Handling
```python
try:
    chrome = PyXA.Application("Google Chrome")
    tab = chrome.current_tab()
    if tab:
        title = tab.title()
        print(f"Current tab: {title}")
    else:
        print("No active tab found")
except Exception as e:
    print(f"Browser automation error: {e}")
```