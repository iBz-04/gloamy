# Chrome JXA recipes

## List all tab URLs (batch)
```javascript
const win = chrome.windows[0];
const urls = win.tabs.url();
JSON.stringify(urls);
```

## Close tabs by pattern (reverse order)
```javascript
const win = chrome.windows[0];
const urls = win.tabs.url();
const toClose = [];
urls.forEach((u, i) => { if (u.includes("youtube")) toClose.push(i); });
toClose.reverse().forEach(i => win.tabs[i].close());
```

## Open a new tab
```javascript
const tab = chrome.Tab({ url: "https://example.com" });
chrome.windows[0].tabs.push(tab);
```

