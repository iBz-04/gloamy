# PyXA Basics (Python for macOS Automation)

## Table of Contents
1. [Mental Model](#mental-model)
2. [Core Patterns](#core-patterns)
3. [Error Handling](#error-handling)
4. [Security](#security)
5. [Performance](#performance)
6. [Comprehensive Examples](#comprehensive-examples)
7. [Brief Examples](#brief-examples)

## Mental Model
PyXA (Python for macOS Automation) is a modern Python wrapper around Apple's Scripting Bridge framework, enabling AppleScript- and JXA-like control over macOS applications. Unlike legacy JXA, PyXA uses Pythonic syntax with snake_case conventions and command chaining.

**Key Concepts:**
- **Application Objects**: References to macOS apps (e.g., `PyXA.Application("Safari")`)
- **Object Hierarchy**: Apps contain windows, documents, and other objects accessible via method chaining
- **Properties vs Methods**: Object attributes use dot notation, actions use method calls
- **Lists**: Bulk operations via `XAList` subclasses for efficient batch processing
- **UI Scripting**: System Events access for non-scriptable app automation

## Core Patterns

### Application Control Pattern
```python
import PyXA
app = PyXA.Application("Safari")
app.activate()  # Bring app to foreground
app.front_window.current_tab.url  # Access nested properties
```

### Command Chaining Pattern
```python
# Chain methods for concise operations
PyXA.Application("Notes").new_note("Title", "Content").show()
```

### List Operations Pattern
```python
# Use XAList for bulk operations
notes = PyXA.Application("Notes").notes()
titles = notes.name()  # Get all note titles at once
```

### UI Scripting Pattern
```python
# Access non-scriptable apps via System Events
window = PyXA.Application("AppName").front_window
buttons = window.buttons()
play_button = buttons.by_object_description("Play")
play_button.click()
```

## Error Handling

### Basic Exception Handling
```python
import PyXA
from PyXA.XAErrors import ApplicationNotFoundError

try:
    safari = PyXA.Application("Safari")
    url = safari.front_window.current_tab.url
    print(f"Current URL: {url}")
except ApplicationNotFoundError:
    print("Safari is not installed or running")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### App State Validation
```python
def safe_get_safari_url():
    try:
        safari = PyXA.Application("Safari")
        if not safari.running():
            safari.launch()
            # Wait for app to be ready
            import time
            time.sleep(2)
        return safari.front_window.current_tab.url
    except Exception as e:
        print(f"Failed to get Safari URL: {e}")
        return None
```

### Permission Error Handling
```python
try:
    mail = PyXA.Application("Mail")
    messages = mail.inboxes()[0].messages()
except Exception as e:
    if "permission" in str(e).lower():
        print("Automation permission denied. Check System Settings > Privacy & Security")
    else:
        print(f"Mail access error: {e}")
```

## Security

### Permission Management
- **Automation Permissions**: Required for app control (System Settings > Privacy & Security > Automation)
- **Accessibility Permissions**: Needed for UI scripting (System Settings > Privacy & Security > Accessibility)
- **Minimal Permissions**: Request only necessary permissions to reduce attack surface

### Input Validation
```python
def safe_create_note(title, content):
    # Validate inputs to prevent injection
    if not isinstance(title, str) or not isinstance(content, str):
        raise ValueError("Title and content must be strings")
    
    # Sanitize content
    safe_content = content.replace("<script>", "").replace("</script>", "")
    
    try:
        PyXA.Application("Notes").new_note(title, safe_content)
    except Exception as e:
        print(f"Failed to create note: {e}")
```

### Code Signing
- Sign scripts for production use with Developer ID certificate
- Use `codesign` command for distribution
- Avoid storing credentials in scripts

## Performance

### Use XAList for Bulk Operations
```python
import PyXA
from timeit import timeit

# Inefficient: Individual operations
def slow_method():
    notes = PyXA.Application("Notes").notes()
    titles = []
    for note in notes:
        titles.append(note.name)
    return titles

# Efficient: Bulk operations
def fast_method():
    notes = PyXA.Application("Notes").notes()
    return notes.name()  # 56x faster for large lists

# Measure performance
slow_time = timeit(slow_method, number=10)
fast_time = timeit(fast_method, number=10)
print(f"Slow: {slow_time:.4f}s, Fast: {fast_time:.4f}s")
```

### Minimize Apple Events
```python
# Bad: Multiple Apple Events
tab = PyXA.Application("Safari").front_window.current_tab
url = tab.url
name = tab.name
print(url, name)

# Good: Single Apple Event with stored reference
tab = PyXA.Application("Safari").front_window.current_tab
print(tab.url, tab.name)
```

### Cache References
```python
# Cache app references for repeated use
safari = PyXA.Application("Safari")

def get_current_url():
    return safari.front_window.current_tab.url  # Reuses cached reference
```

## Comprehensive Examples

### Example 1: Safari Tab Manager
```python
#!/usr/bin/env python
import PyXA

class SafariTabManager:
    def __init__(self):
        try:
            self.safari = PyXA.Application("Safari")
            if not self.safari.running():
                self.safari.launch()
        except Exception as e:
            print(f"Failed to initialize Safari: {e}")
            self.safari = None
    
    def get_all_tab_info(self):
        """Get URLs and titles of all tabs across all windows"""
        if not self.safari:
            return []
        
        tab_info = []
        try:
            for window in self.safari.windows():
                for tab in window.tabs():
                    tab_info.append({
                        'url': tab.url,
                        'title': tab.name,
                        'window_index': window.index
                    })
        except Exception as e:
            print(f"Error getting tab info: {e}")
        
        return tab_info
    
    def close_duplicate_tabs(self):
        """Close duplicate URLs, keeping the first occurrence"""
        if not self.safari:
            return 0
        
        closed_count = 0
        seen_urls = set()
        
        try:
            # Collect all tabs first to avoid modification during iteration
            all_tabs = []
            for window in self.safari.windows():
                for tab in window.tabs():
                    all_tabs.append(tab)
            
            # Close duplicates
            for tab in all_tabs:
                if tab.url in seen_urls:
                    tab.close()
                    closed_count += 1
                else:
                    seen_urls.add(tab.url)
                    
        except Exception as e:
            print(f"Error closing duplicates: {e}")
        
        return closed_count
    
    def create_session_note(self):
        """Save current tab session to Notes app"""
        if not self.safari:
            return
        
        try:
            tabs = self.get_all_tab_info()
            content = "Safari Session:\n\n"
            
            for i, tab in enumerate(tabs, 1):
                content += f"{i}. [{tab['title']}]({tab['url']})\n"
            
            PyXA.Application("Notes").new_note("Safari Session", content).show()
            
        except Exception as e:
            print(f"Error creating session note: {e}")

# Usage
manager = SafariTabManager()
print("All tabs:", manager.get_all_tab_info())
closed = manager.close_duplicate_tabs()
print(f"Closed {closed} duplicate tabs")
manager.create_session_note()
```

### Example 2: Mail Processor with Reminders Integration
```python
#!/usr/bin/env python
import PyXA
from datetime import datetime, timedelta

class MailReminderProcessor:
    def __init__(self):
        try:
            self.mail = PyXA.Application("Mail")
            self.reminders = PyXA.Application("Reminders")
        except Exception as e:
            print(f"Failed to initialize apps: {e}")
            self.mail = None
            self.reminders = None
    
    def process_unread_messages(self):
        """Create reminders for unread messages requiring follow-up"""
        if not self.mail or not self.reminders:
            return 0
        
        reminder_count = 0
        
        try:
            # Get all unread messages
            unread_messages = []
            for account in self.mail.accounts():
                for mailbox in account.mailboxes():
                    if "INBOX" in mailbox.name.upper():
                        unread_messages.extend(mailbox.messages().by_read_status(False))
            
            for message in unread_messages[:10]:  # Limit to 10 for performance
                # Check if message needs follow-up
                if self._needs_followup(message):
                    self._create_followup_reminder(message)
                    reminder_count += 1
                    
        except Exception as e:
            print(f"Error processing messages: {e}")
        
        return reminder_count
    
    def _needs_followup(self, message):
        """Determine if message needs follow-up based on content analysis"""
        subject = message.subject or ""
        sender = message.sender or ""
        
        # Simple heuristics - customize as needed
        followup_keywords = ['follow up', 'action required', 'please respond', 'urgent']
        important_senders = ['boss@company.com', 'client@domain.com']
        
        subject_lower = subject.lower()
        sender_lower = sender.lower()
        
        return (
            any(keyword in subject_lower for keyword in followup_keywords) or
            any(important in sender_lower for important in important_senders)
        )
    
    def _create_followup_reminder(self, message):
        """Create a reminder for message follow-up"""
        try:
            subject = message.subject or "No Subject"
            sender = message.sender or "Unknown Sender"
            
            title = f"Follow up: {subject}"
            notes = f"From: {sender}\nOriginal message: {message.id()}"
            
            # Create reminder due in 2 days
            due_date = datetime.now() + timedelta(days=2)
            
            # Get or create "Follow-ups" list
            followup_list = None
            for list_obj in self.reminders.lists():
                if list_obj.name == "Follow-ups":
                    followup_list = list_obj
                    break
            
            if not followup_list:
                followup_list = self.reminders.new_list("Follow-ups")
            
            followup_list.reminders().push({
                'name': title,
                'body': notes,
                'due_date': due_date
            })
            
        except Exception as e:
            print(f"Error creating reminder for message {message.id()}: {e}")

# Usage
processor = MailReminderProcessor()
created = processor.process_unread_messages()
print(f"Created {created} follow-up reminders")
```

## Brief Examples

### Basic App Control
```python
# Launch and activate app
PyXA.Application("Calculator").activate()

# Check if app is running
running = PyXA.Application("Safari").running()

# Get frontmost app
front_app = PyXA.current_application()
```

### File Operations
```python
# Create new TextEdit document
textedit = PyXA.Application("TextEdit")
doc = textedit.new_document("Hello World")

# Save document
doc.save_in(PyXA.Path("/Users/user/Desktop/hello.txt"))
```

### Finder Operations
```python
finder = PyXA.Application("Finder")

# Get desktop files
desktop_files = finder.desktop.files()

# Create new folder
new_folder = finder.desktop.folders().push("New Folder")

# Move files
file_to_move = finder.desktop.files().by_name("old.txt")
file_to_move.move_to(new_folder)
```

### Calendar Events
```python
calendar_app = PyXA.Application("Calendar")

# Create new event
event = calendar_app.calendars()[0].events().push({
    'summary': 'Meeting',
    'start_date': datetime(2024, 1, 15, 10, 0),
    'end_date': datetime(2024, 1, 15, 11, 0)
})

# Get today's events
today_events = calendar_app.events().by_start_date(datetime.now().date())
```

### Clipboard Operations
```python
# Copy text to clipboard
PyXA.XAClipboard().set_text("Hello from PyXA")

# Get clipboard content
clipboard_text = PyXA.XAClipboard().text

# Copy image
image = PyXA.XAImage("/path/to/image.jpg")
PyXA.XAClipboard().set_image(image)
```

### UI Scripting
```python
# Click system dialog button
system_events = PyXA.Application("System Events")
dialog = system_events.processes().by_name("App Name").windows()[0]
ok_button = dialog.buttons().by_name("OK")
ok_button.click()
```

### Notifications
```python
# Show notification
notification = PyXA.XANotification()
notification.title = "Task Complete"
notification.subtitle = "Your automation finished"
notification.body = "All items processed successfully"
notification.show()
```

### Speech Recognition
```python
# Basic speech recognition
recognizer = PyXA.XASpeechRecognizer()
recognizer.start()

# Listen for specific commands
detector = PyXA.XACommandDetector()
detector.commands = ["open safari", "close window"]
detector.start()
```

### Image Processing
```python
# Load and manipulate images
image = PyXA.XAImage("/path/to/photo.jpg")
resized = image.resize(800, 600)
flipped = resized.flip_horizontally()
flipped.save("/path/to/modified.jpg")
```

### Menu Bar Extras
```python
# Add item to menu bar
menu_bar = PyXA.XAMenuBar()
menu_item = menu_bar.add_menu_item("PyXA Action")
menu_item.action = lambda: print("Menu clicked!")
```