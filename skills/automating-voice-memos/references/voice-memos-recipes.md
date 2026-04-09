# Voice Memos Recipes

## Batch export (data-first)
```javascript
ObjC.import('Foundation');
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const fm = $.NSFileManager.defaultManager;
const home = fm.homeDirectoryForCurrentUser.path.js;
const root = home + "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings";
const dbPath = root + "/CloudRecordings.db";
const outDir = home + "/Desktop/Exported Voice Memos";
if (!fm.fileExistsAtPath(outDir)) fm.createDirectoryAtPathWithIntermediateDirectoriesAttributesError(outDir, true, null, null);

const sql = `SELECT ZPATH, ZCUSTOMLABEL, ZDATE FROM ZCLOUDRECORDING WHERE ZTRASHEDDATE IS NULL;`;
const cmd = `sqlite3 -separator "|" "${dbPath}" "${sql}"`;
const rows = app.doShellScript(cmd).split('\r');

rows.forEach(line => {
  if (!line.trim()) return;
  const [file, titleRaw, zdateRaw] = line.split('|');
  const date = new Date((parseFloat(zdateRaw) + 978307200) * 1000);
  const dateStr = date.toISOString().split('T');
  const title = (titleRaw || "Untitled").replace(/[\/\\:]/g, "-");
  let dest = `${outDir}/${dateStr} ${title}.m4a`;
  let c = 1;
  while (fm.fileExistsAtPath(dest)) dest = `${outDir}/${dateStr} ${title} ${c++}.m4a`;
  const src = `${root}/${file}`;
  if (fm.fileExistsAtPath(src)) fm.copyItemAtPathToPathError(src, dest, null);
});
```

## Start/stop recording via shortcuts
```javascript
const se = Application("System Events");
const vm = Application("Voice Memos");
vm.activate();
delay(0.2);
se.keystroke("n", {using: "command down"}); // start
// ... wait as needed ...
se.keystroke(".", {using: "command down"}); // stop
```

## Transcript scrape (outline)
1) Activate Voice Memos, `Cmd+F`, type recording name, Enter.  
2) Open transcript via View menu or transcript button.  
3) Use a recursive UI search to find `AXTextArea` and read `.value()`.  
4) Save text next to the exported audio. Example write:
```javascript
const app = Application.currentApplication();
app.includeStandardAdditions = true;
const text = transcriptText; // result of AXTextArea.value()
const out = `${outDir}/${safeTitle}.txt`;
app.doShellScript(`printf %s ${app.doShellScript("python3 - <<'PY'\nimport sys,json\nprint(json.dumps(sys.stdin.read()))\nPY <<< " + JSON.stringify(text))} > ${out}`);
```

## External trim/enhance workflow
1) Export recording (data workflow above).  
2) Run `ffmpeg` via `doShellScript` for silence removal or trimming.  
3) Optional: re-import by simulating drag/drop or opening in Finder, then delete the original via DB/UI if needed.  
