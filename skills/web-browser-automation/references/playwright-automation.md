# Playwright Browser Automation

## Installation
```bash
pip install playwright
playwright install  # Downloads browser binaries
```

## Basic Usage
```python
from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()
    page.goto("https://example.com")
    title = page.title()
    print(f"Page title: {title}")
    browser.close()
```

## Browser Types
```python
# Chromium (Chrome/Edge/Chromium)
browser = p.chromium.launch()

# Firefox
browser = p.firefox.launch()

# WebKit (Safari)
browser = p.webkit.launch()
```

## Advanced Launch Options
```python
browser = p.chromium.launch(
    headless=False,  # Show browser window
    slow_mo=1000,   # Slow down operations by 1 second
    args=[
        '--disable-blink-features=AutomationControlled',
        '--no-sandbox',
        '--disable-dev-shm-usage'
    ]
)
```

## Page Navigation and Interaction
```python
page = browser.new_page()

# Navigation
page.goto("https://example.com")
page.go_back()
page.go_forward()
page.reload()

# Auto-waiting interactions
page.click("text=Get Started")  # Waits for element automatically
page.fill("#username", "user@example.com")
page.press("#username", "Enter")

# Wait for specific conditions
page.wait_for_selector(".results")
page.wait_for_load_state("networkidle")
```

## Element Selection and Manipulation
```python
# CSS selectors
button = page.locator("#submit-button")
button.click()

# Text-based selection
page.click("text=Sign In")

# XPath
page.click("//button[contains(text(), 'Submit')]")

# Multiple elements
links = page.locator("a").all()
for link in links:
    print(link.get_attribute("href"))
```

## Screenshots and Recording
```python
# Screenshot full page
page.screenshot(path="fullpage.png")

# Screenshot specific element
element = page.locator(".content")
element.screenshot(path="element.png")

# Video recording
context = browser.new_context(record_video_dir="videos/")
page = context.new_page()
# ... automation ...
context.close()  # Saves video
```

## Network Interception
```python
# Intercept and modify requests
def handle_request(request):
    if "analytics" in request.url:
        request.abort()  # Block analytics
    elif "api.example.com" in request.url:
        # Modify API calls
        request.continue_(headers={**request.headers, "X-Custom": "value"})
    else:
        request.continue_()

page.route("**/*", handle_request)
```

## File Upload and Download
```python
# File upload
page.set_input_files('input[type="file"]', 'path/to/file.pdf')

# Handle downloads
with page.expect_download() as download_info:
    page.click('a[download]')
download = download_info.value
download.save_as("downloads/file.pdf")
```

## Mobile Emulation
```python
# iPhone simulation
context = browser.new_context(
    viewport={"width": 375, "height": 667},
    user_agent="Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15"
)
page = context.new_page()
```

## Cross-Browser Testing
```python
browsers = ["chromium", "firefox", "webkit"]
results = {}

for browser_name in browsers:
    browser_type = getattr(p, browser_name)
    browser = browser_type.launch()

    try:
        page = browser.new_page()
        page.goto("https://httpbin.org/user-agent")
        ua = page.text_content("body")
        results[browser_name] = f"SUCCESS: {ua[:50]}..."
    except Exception as e:
        results[browser_name] = f"FAILED: {e}"
    finally:
        browser.close()

print("Cross-browser results:", results)
```

## Error Handling and Timeouts
```python
try:
    page.goto("https://slowsite.com", timeout=10000)
    page.click(".might-not-exist", timeout=5000)
except Exception as e:
    print(f"Operation failed: {e}")
    # Take screenshot for debugging
    page.screenshot(path="error_screenshot.png")
```

## Configuration Files
```python
# playwright.config.py
import os
from playwright.sync_api import expect

os.environ["PWDEBUG"] = "1"  # Enable debug mode

expect.set_options(timeout=10000)  # Global timeout

# Configuration for different environments
config = {
    "chromium": {
        "headless": True,
        "args": ["--no-sandbox"]
    },
    "firefox": {
        "headless": True
    },
    "webkit": {
        "headless": True
    }
}
```