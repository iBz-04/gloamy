# Chrome JXA advanced patterns

## Execute (DOM) with title tunnel
```javascript
function execWithTitleTunnel(tab, jsExpr) {
  const prefix = "||JXA||";
  const original = tab.title();
  const code = `(function(){try{document.title='${prefix}'+(${jsExpr});}catch(e){document.title='${prefix}ERROR:'+e.message;}})()`;
  tab.execute({ javascript: code });
  let tries = 20;
  while (tries-- > 0) {
    const t = tab.title();
    if (t.startsWith(prefix)) {
      const result = t.slice(prefix.length);
      tab.execute({ javascript: `document.title='${original.replace(/'/g, "\\'")}'` });
      return result;
    }
    delay(0.1);
  }
  return null;
}
```

## Incognito spawn (shell)
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
app.doShellScript("open -na 'Google Chrome' --args --incognito 'about:blank'");
```

## Wait for load
```javascript
function waitForLoad(tab, timeout = 50) {
  while (timeout-- > 0) {
    if (!tab.loading()) return true;
    delay(0.2);
  }
  throw new Error("Timeout waiting for load");
}
```

