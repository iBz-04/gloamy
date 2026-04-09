# Browser Automation Workflows

## Workflow 1: Multi-Browser Tab Management (PyXA)

**Objective**: Synchronize tabs across Chrome, Edge, and Brave browsers for research workflows.

```python
import PyXA
from time import sleep

def sync_research_tabs():
    # Initialize browser applications
    chrome = PyXA.Application("Google Chrome")
    edge = PyXA.Application("Microsoft Edge")
    brave = PyXA.Application("Brave Browser")

    browsers = [chrome, edge, brave]

    # Open research URLs in each browser
    research_urls = [
        "https://scholar.google.com",
        "https://arxiv.org",
        "https://github.com"
    ]

    for browser in browsers:
        if not browser.frontmost:
            browser.frontmost = True
            sleep(0.5)

        # Create new window for research session
        research_window = browser.new_window()

        # Open research tabs
        for url in research_urls:
            research_window.new_tab(url)
            sleep(0.3)

        # Organize tabs by pinning important ones
        tabs = research_window.tabs()
        if len(tabs) > 0:
            # Pin first tab (Google Scholar)
            tabs[0].execute_javascript("javascript:void(0)")  # Focus tab

    # Create summary report
    summary = {
        "chrome_tabs": len(chrome.windows()[0].tabs()) if chrome.windows() else 0,
        "edge_tabs": len(edge.windows()[0].tabs()) if edge.windows() else 0,
        "brave_tabs": len(brave.windows()[0].tabs()) if brave.windows() else 0
    }

    return summary

# Execute workflow
result = sync_research_tabs()
print(f"Research session initialized: {result}")
```

## Workflow 2: Automated Research and Data Collection (PyXA)

**Objective**: Extract and organize research data from multiple browser tabs using JavaScript execution and clipboard integration.

```python
import PyXA
import json
from datetime import datetime

def collect_research_data():
    chrome = PyXA.Application("Google Chrome")

    if not chrome.windows():
        print("No Chrome windows open")
        return {}

    front_window = chrome.windows()[0]
    research_data = {
        "timestamp": datetime.now().isoformat(),
        "tabs": [],
        "bookmarks": []
    }

    # Collect tab information
    tabs = front_window.tabs()
    for tab in tabs:
        if not tab.loading:  # Only process loaded tabs
            tab_info = {
                "title": tab.title,
                "url": str(tab.url),
                "id": tab.id
            }

            # Extract page metadata using JavaScript
            try:
                meta_description = tab.execute_javascript("""
                    const meta = document.querySelector('meta[name="description"]');
                    return meta ? meta.getAttribute('content') : null;
                """)
                tab_info["description"] = meta_description
            except:
                tab_info["description"] = None

            research_data["tabs"].append(tab_info)

    # Collect relevant bookmarks
    try:
        bookmarks_bar = chrome.bookmarks_bar
        research_bookmarks = bookmarks_bar.bookmark_items()

        for bookmark in research_bookmarks:
            if any(keyword in bookmark.title.lower() for keyword in
                  ["research", "paper", "study", "science"]):
                research_data["bookmarks"].append({
                    "title": bookmark.title,
                    "url": str(bookmark.url)
                })
    except Exception as e:
        print(f"Error collecting bookmarks: {e}")

    # Save to organized format
    filename = f"research_session_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    with open(filename, 'w') as f:
        json.dump(research_data, f, indent=2)

    return research_data

# Execute data collection
data = collect_research_data()
print(f"Collected data from {len(data.get('tabs', []))} tabs and {len(data.get('bookmarks', []))} bookmarks")
```

## Workflow 3: Cross-Browser Testing Suite (Playwright)

**Objective**: Run automated tests across multiple browsers with comprehensive reporting.

```python
from playwright.sync_api import sync_playwright
import json
from datetime import datetime

def run_cross_browser_tests():
    """Run automated tests across Chrome, Firefox, and Safari"""
    test_results = {
        "timestamp": datetime.now().isoformat(),
        "tests": []
    }

    browsers_to_test = ["chromium", "firefox", "webkit"]

    with sync_playwright() as p:
        for browser_name in browsers_to_test:
            browser_type = getattr(p, browser_name)

            try:
                browser = browser_type.launch()
                context = browser.new_context()
                page = context.new_page()

                # Test 1: Basic navigation
                start_time = datetime.now()
                page.goto("https://httpbin.org/html")
                load_time = (datetime.now() - start_time).total_seconds()

                # Test 2: JavaScript execution
                title = page.evaluate("document.title")

                # Test 3: Element interaction
                page.click("text=Sample")
                page.wait_for_selector("h1")

                # Test 4: Screenshot
                screenshot_path = f"screenshot_{browser_name}.png"
                page.screenshot(path=screenshot_path)

                test_results["tests"].append({
                    "browser": browser_name,
                    "status": "passed",
                    "load_time": load_time,
                    "title": title,
                    "screenshot": screenshot_path
                })

                browser.close()

            except Exception as e:
                test_results["tests"].append({
                    "browser": browser_name,
                    "status": "failed",
                    "error": str(e)
                })

    # Save results
    with open("cross_browser_test_results.json", "w") as f:
        json.dump(test_results, f, indent=2)

    return test_results

# Run the test suite
results = run_cross_browser_tests()
passed_tests = sum(1 for test in results["tests"] if test["status"] == "passed")
print(f"Test Results: {passed_tests}/{len(results['tests'])} tests passed")
```

## Workflow 4: Web Scraping and Data Extraction (Puppeteer)

**Objective**: Extract structured data from web pages with error handling and rate limiting.

```javascript
const puppeteer = require('puppeteer');
const fs = require('fs');

async function scrapeProductData() {
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  const page = await browser.new_page();
  const scrapedData = [];

  try {
    // Navigate to target page
    await page.goto('https://example.com/products', {
      waitUntil: 'networkidle0',
      timeout: 30000
    });

    // Wait for products to load
    await page.waitForSelector('.product-item', { timeout: 10000 });

    // Extract product data
    const products = await page.$$eval('.product-item', items => {
      return items.map(item => {
        const title = item.querySelector('.product-title')?.textContent?.trim();
        const price = item.querySelector('.price')?.textContent?.trim();
        const link = item.querySelector('a')?.href;

        return {
          title,
          price,
          link,
          timestamp: new Date().toISOString()
        };
      }).filter(product => product.title && product.price);
    });

    scrapedData.push(...products);

    // Handle pagination if present
    const nextButton = await page.$('a[aria-label="Next page"]');
    if (nextButton) {
      await nextButton.click();
      await page.waitForTimeout(2000); // Rate limiting

      // Recursively scrape next page
      const nextPageData = await page.$$eval('.product-item', items => {
        return items.map(item => ({
          title: item.querySelector('.product-title')?.textContent?.trim(),
          price: item.querySelector('.price')?.textContent?.trim(),
          link: item.querySelector('a')?.href,
          timestamp: new Date().toISOString()
        })).filter(product => product.title && product.price);
      });

      scrapedData.push(...nextPageData);
    }

  } catch (error) {
    console.error('Scraping error:', error.message);
  } finally {
    await browser.close();
  }

  // Save data
  fs.writeFileSync('product_data.json', JSON.stringify({
    scrape_timestamp: new Date().toISOString(),
    total_products: scrapedData.length,
    products: scrapedData
  }, null, 2));

  return scrapedData;
}

// Run the scraper
scrapeProductData().then(data => {
  console.log(`Scraped ${data.length} products`);
}).catch(console.error);
```