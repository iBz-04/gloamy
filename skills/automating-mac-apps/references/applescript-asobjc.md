# AppleScriptObjC (ASObjC) essentials

Use ASObjC when AppleScript needs regex, JSON, or fast file IO.

## Enable ASObjC
```applescript
use AppleScript version "2.4"
use framework "Foundation"
use scripting additions
```

## Regex search
```applescript
on regexFind(sourceText, pattern)
  set str to current application's NSString's stringWithString:sourceText
  set regex to current application's NSRegularExpression's regularExpressionWithPattern:pattern options:0 |error|:(missing value)
  set matches to regex's matchesInString:str options:0 range:{location:0, |length|:str's |length|()}
  set resultList to {}
  repeat with aMatch in matches
    set matchRange to aMatch's range()
    set end of resultList to (str's substringWithRange:matchRange) as text
  end repeat
  return resultList
end regexFind
```

## JSON parse/stringify
```applescript
on parseJSON(jsonStr)
  set str to current application's NSString's stringWithString:jsonStr
  set d to str's dataUsingEncoding:(current application's NSUTF8StringEncoding)
  set {res, err} to current application's NSJSONSerialization's JSONObjectWithData:d options:0 |error|:(reference)
  if res is missing value then error (err's localizedDescription() as text)
  return res as list
end parseJSON

on stringifyJSON(asListOrRecord)
  if class of asListOrRecord is record then
    set cocoaObj to current application's NSDictionary's dictionaryWithDictionary:asListOrRecord
  else
    set cocoaObj to current application's NSArray's arrayWithArray:asListOrRecord
  end if
  set {d, err} to current application's NSJSONSerialization's dataWithJSONObject:cocoaObj options:0 |error|:(reference)
  if d is missing value then error (err's localizedDescription() as text)
  set str to current application's NSString's alloc()'s initWithData:d encoding:(current application's NSUTF8StringEncoding)
  return str as text
end stringifyJSON
```

## Atomic file write
```applescript
on writeTextToFile(theText, thePath)
  set str to current application's NSString's stringWithString:theText
  set pth to current application's NSString's stringWithString:thePath
  set expPath to pth's stringByExpandingTildeInPath()
  set {res, err} to str's writeToFile:expPath atomically:true encoding:(current application's NSUTF8StringEncoding) |error|:(reference)
  if res is false then error (err's localizedDescription() as text)
end writeTextToFile
```

