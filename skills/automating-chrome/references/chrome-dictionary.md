# Chrome dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
window 1                            chrome.windows[0]
active tab                          chrome.windows[0].activeTab
URL of active tab                   chrome.windows[0].activeTab.url()
execute javascript "..."            tab.execute({ javascript: "..." })
make new tab with URL               chrome.Tab({ url: "..." }) -> push()
```

Notes:
- Collections are specifiers; call methods to read values.
- Use batch property reads (e.g., tabs.url()).

