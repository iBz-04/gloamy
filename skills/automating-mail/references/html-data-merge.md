# HTML data merge

## Merge JSON data into HTML template
```javascript
const Mail = Application("Mail");

function renderTemplate(tpl, data) {
  return tpl.replace(/\{\{(\w+)\}\}/g, (_, k) => data[k] ?? "");
}

const template = "<html><body><h1>Report for {{name}}</h1><p>Total: {{total}}</p></body></html>";
const data = { name: "Acme", total: "$1,234" };

const msg = Mail.OutgoingMessage({ subject: "HTML Report", visible: false });
Mail.outgoingMessages.push(msg);
msg.htmlContent = renderTemplate(template, data);
msg.visible = true;
```

