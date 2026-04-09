# JXA Safari recipes

## List all tabs
```javascript
const safari = Application("Safari");
const tabs = [];
safari.windows().forEach(win => {
  win.tabs().forEach(tab => tabs.push({ title: tab.name(), url: tab.url() }));
});
JSON.stringify({ count: tabs.length, tabs });
```
