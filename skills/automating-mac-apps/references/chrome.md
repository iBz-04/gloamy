# JXA Chrome recipes

## Required setting
- In Chrome: View -> Developer -> Allow JavaScript from Apple Events

## List tabs
```javascript
const chrome = Application("Google Chrome");
const tabs = [];
chrome.windows().forEach(win => {
  win.tabs().forEach(tab => tabs.push({ title: tab.title(), url: tab.url() }));
});
JSON.stringify({ count: tabs.length, tabs });
```
