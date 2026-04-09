# Puppeteer Node.js Automation

## Installation
```bash
npm install puppeteer
```

## Basic Usage
```javascript
const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch();
  const page = await puppeteer.new_page();
  await page.goto('https://example.com');
  const title = await page.title();
  console.log(`Page title: ${title}`);
  await browser.close();
})();
```

## Launch Options
```javascript
const browser = await puppeteer.launch({
  headless: 'new',  // Modern headless mode
  args: [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--disable-dev-shm-usage'
  ],
  defaultViewport: {
    width: 1280,
    height: 720
  }
});
```

## Page Navigation
```javascript
await page.goto('https://example.com', {
  waitUntil: 'networkidle0',
  timeout: 30000
});

await page.goBack();
await page.goForward();
await page.reload();
```

## Element Interaction
```javascript
// Click elements
await page.click('button.primary');
await page.click('#submit-btn');

// Fill forms
await page.type('#username', 'user@example.com');
await page.type('#password', 'securepassword');

// Select from dropdowns
await page.select('#country', 'US');

// Checkboxes and radio buttons
await page.check('#agree-terms');
await page.uncheck('#newsletter');
```

## Content Extraction
```javascript
// Get text content
const text = await page.$eval('h1', el => el.textContent);

// Extract multiple elements
const links = await page.$$eval('a', anchors =>
  anchors.map(a => ({ text: a.textContent, href: a.href }))
);

// Get page HTML
const html = await page.content();
```

## Screenshots and PDFs
```javascript
// Full page screenshot
await page.screenshot({
  path: 'screenshot.png',
  fullPage: true
});

// Element screenshot
const element = await page.$('.content');
await element.screenshot({ path: 'element.png' });

// Generate PDF
await page.pdf({
  path: 'page.pdf',
  format: 'A4',
  printBackground: true,
  margin: {
    top: '1cm',
    right: '1cm',
    bottom: '1cm',
    left: '1cm'
  }
});
```

## JavaScript Execution
```javascript
// Execute JavaScript in page context
const dimensions = await page.evaluate(() => {
  return {
    width: document.documentElement.clientWidth,
    height: document.documentElement.clientHeight,
    deviceScaleFactor: window.devicePixelRatio
  };
});

console.log('Page dimensions:', dimensions);

// Pass data to page context
await page.evaluate((data) => {
  console.log('Data from Node.js:', data);
}, { message: 'Hello from Puppeteer' });
```

## Network Interception
```javascript
// Block unwanted resources
await page.setRequestInterception(true);
page.on('request', (request) => {
  const resourceType = request.resourceType();
  if (resourceType === 'image' || resourceType === 'media') {
    request.abort();
  } else {
    request.continue();
  }
});

// Monitor network activity
page.on('response', response => {
  console.log(`${response.status()} ${response.url()}`);
});
```

## File Upload and Download
```javascript
// File upload
const input = await page.$('input[type="file"]');
await input.uploadFile('/path/to/file.pdf');

// Handle downloads
const [download] = await Promise.all([
  page.waitForEvent('download'),
  page.click('a[download]')
]);
await download.saveAs('/tmp/downloaded-file.pdf');
```

## Authentication and Cookies
```javascript
// Set authentication
await page.authenticate({
  username: 'user',
  password: 'pass'
});

// Cookie management
await page.setCookie({
  name: 'session',
  value: 'abc123',
  domain: 'example.com'
});

const cookies = await page.cookies();
console.log('Cookies:', cookies);
```

## Mobile Emulation
```javascript
// iPhone simulation
await page.setViewport({
  width: 375,
  height: 667,
  deviceScaleFactor: 2,
  isMobile: true,
  hasTouch: true
});

await page.setUserAgent(
  'Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15'
);
```

## Performance Monitoring
```javascript
// Measure page load time
const startTime = Date.now();
await page.goto('https://example.com');
const loadTime = Date.now() - startTime;
console.log(`Page loaded in ${loadTime}ms`);

// Monitor performance metrics
const metrics = await page.metrics();
console.log('Performance metrics:', metrics);

// Capture traces
await page.tracing.start({ path: 'trace.json' });
// ... perform actions ...
await page.tracing.stop();
```

## Error Handling
```javascript
try {
  await page.goto('https://example.com', { timeout: 10000 });
  await page.click('.might-not-exist', { timeout: 5000 });
} catch (error) {
  console.error('Operation failed:', error.message);

  // Take screenshot for debugging
  await page.screenshot({ path: 'error-screenshot.png' });
}
```

## Browser Contexts and Pages
```javascript
// Multiple pages
const page1 = await browser.newPage();
const page2 = await browser.newPage();

// Incognito context
const context = await browser.createIncognitoBrowserContext();
const privatePage = await context.newPage();

// Session isolation
const session1 = await browser.newContext();
const session2 = await browser.newContext();
```