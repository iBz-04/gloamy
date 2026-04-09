# Form automation basics (Chrome/Chromium)

## Set input value (React/Vue-safe)
```javascript
function setInputValue(tab, selector, value) {
  const js = `
    (function() {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return "NOT_FOUND";
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
      setter.call(el, ${JSON.stringify(value)});
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return "OK";
    })();
  `;
  return tab.execute({ javascript: js });
}
```

## Click a button
```javascript
function clickButton(tab, selector) {
  const js = `
    (function() {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return "NOT_FOUND";
      el.click();
      return "OK";
    })();
  `;
  return tab.execute({ javascript: js });
}
```

## Select dropdown
```javascript
function setSelectValue(tab, selector, value) {
  const js = `
    (function() {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return "NOT_FOUND";
      el.value = ${JSON.stringify(value)};
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return "OK";
    })();
  `;
  return tab.execute({ javascript: js });
}
```

## Wait for element (poll)
```javascript
function waitForElement(tab, selector, timeoutSecs = 10) {
  const deadline = Date.now() + timeoutSecs * 1000;
  while (Date.now() < deadline) {
    const exists = tab.execute({ javascript: `document.querySelector(${JSON.stringify(selector)}) !== null` });
    if (exists) return true;
    delay(0.2);
  }
  throw new Error("Timeout waiting for " + selector);
}
```

