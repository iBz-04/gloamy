#!/usr/bin/osascript
(*
UI-only export for the currently selected Voice Memos recording.
Avoids reading the database/files directly (no Full Disk Access required).
Requirements: Voice Memos open and a recording selected; Accessibility enabled for the runner (Terminal/Script Editor).

Usage (defaults to Desktop/voice-memo-export.m4a):
  osascript export_selected_recording_via_ui.applescript
  osascript export_selected_recording_via_ui.applescript "/path/to/dir" "my-recording"
*)

on run argv
    set targetFolder to POSIX path of (path to desktop)
    set baseName to "voice-memo-export"
    if (count of argv) ≥ 1 then set targetFolder to item 1 of argv
    if (count of argv) ≥ 2 then set baseName to item 2 of argv

    tell application "Voice Memos" to activate
    delay 0.3

    tell application "System Events"
        if not (exists process "VoiceMemos") then error "Voice Memos process not found. Launch it first."
        tell process "VoiceMemos"
            set frontmost to true
            my openExportMenu(menu bar 1)
        end tell
        my fillSavePanel(targetFolder, baseName)
    end tell
end run

on openExportMenu(mb)
    tell application "System Events"
        tell mb
            tell menu bar item "File"
                tell menu "File"
                    if exists menu item "Export…" then
                        click menu item "Export…"
                    else if exists menu item "Export..." then
                        click menu item "Export..."
                    else
                        set candidates to every menu item whose name begins with "Export"
                        if candidates is not {} then
                            click item 1 of candidates
                        else
                            keystroke "e" using {command down, shift down} -- fallback shortcut (may vary by macOS version)
                        end if
                    end if
                end tell
            end tell
        end tell
    end tell
end openExportMenu

on fillSavePanel(targetFolder, baseName)
    tell application "System Events"
        tell process "VoiceMemos"
            set saveSheet to missing value
            repeat 30 times
                if exists sheet 1 of window 1 then
                    set saveSheet to sheet 1 of window 1
                    exit repeat
                else if exists window 1 then
                    try
                        if (subrole of window 1 is "AXDialog") then
                            set saveSheet to window 1
                            exit repeat
                        end if
                    end try
                end if
                delay 0.2
            end repeat
            if saveSheet is missing value then error "Save dialog not found. Ensure a recording is selected."

            if exists text field 1 of saveSheet then set value of text field 1 of saveSheet to baseName

            keystroke "g" using {command down, shift down}
            delay 0.2
            try
                tell sheet 1 of saveSheet
                    if exists text field 1 then
                        set value of text field 1 to targetFolder
                        keystroke return
                    end if
                end tell
            end try
            delay 0.3

            try
                if exists button "Save" of saveSheet then
                    click button "Save" of saveSheet
                else if exists button "Export" of saveSheet then
                    click button "Export" of saveSheet
                end if
            end try
        end tell
    end tell
end fillSavePanel
