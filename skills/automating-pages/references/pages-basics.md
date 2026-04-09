# Pages JXA basics

## Bootstrapping
```javascript
'use strict';
const pages = Application("Pages");
pages.includeStandardAdditions = true;
if (!pages.running()) pages.launch();
pages.activate();
```

## Create from template
```javascript
const template = pages.templates["Blank"];
const docSpec = pages.Document({ documentTemplate: template, name: "Automated Report" });
const doc = docSpec.make();
```

## Open and save
```javascript
const doc = pages.open(Path("/Users/you/Documents/Report.pages"));
doc.save({ in: Path("/Users/you/Documents/Report_Final.pages") });
```

