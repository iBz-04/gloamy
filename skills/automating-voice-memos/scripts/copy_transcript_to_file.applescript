#!/usr/bin/osascript
(*
Copy the transcript of the currently selected Voice Memos recording to a file.
UI-only: avoids DB/files; requires Accessibility. Voice Memos must be open with a recording selected.

Usage:
  osascript copy_transcript_to_file.applescript "/path/to/output.txt"
Defaults:
  ~/Desktop/voice-memo-transcript.txt
*)

on run argv
    set targetFile to (POSIX path of (path to desktop)) & "voice-memo-transcript.txt"
    if (count of argv) ≥ 1 then set targetFile to item 1 of argv

    tell application "Voice Memos" to activate
    delay 0.4

    tell application "System Events"
        if not (exists process "VoiceMemos") then error "Voice Memos process not found. Launch it first."
        tell process "VoiceMemos"
            set frontmost to true

            -- Try to show transcript pane (button with transcript icon/label, may vary by macOS version).
            try
                if exists (first button of toolbar 1 of window 1 whose description contains "Transcript") then
                    click (first button of toolbar 1 of window 1 whose description contains "Transcript")
                    delay 0.2
                end if
            end try

            -- Attempt to focus transcript area; element hierarchy varies by macOS version.
            try
                if exists scroll area 1 of window 1 then
                    tell scroll area 1 of window 1
                        if exists static text 1 then
                            click static text 1
                        end if
                    end tell
                end if
            end try

            -- Select all + copy.
            keystroke "a" using command down
            delay 0.05
            keystroke "c" using command down
        end tell
    end tell

    set transcriptText to the clipboard
    if transcriptText is missing value or transcriptText is "" then
        error "No transcript text found in clipboard. Ensure transcript is visible and selected."
    end if

    set outFile to POSIX file targetFile as text
    set fh to open for access outFile with write permission
    try
        set eof fh to 0
        write transcriptText to fh
    on error errMsg number errNum
        close access fh
        error errMsg number errNum
    end try
    close access fh

    return "Transcript saved to " & targetFile
end run
