#!/usr/bin/env osascript -l JavaScript
/**
 * create_reminder.js - Create a reminder in Apple Reminders via JXA
 *
 * Usage:
 *   osascript -l JavaScript create_reminder.js
 *
 * This script creates a reminder with a timed alert. It uses a fallback
 * approach to find an available list since list names vary by system
 * (e.g., "Reminders", "Inbox", or localized names).
 */

function getList(app, preferredName) {
  // Try to get list by preferred name first
  try {
    const list = app.lists.byName(preferredName);
    list.name(); // Verify it exists
    return list;
  } catch (e) {
    // Fall back to first available list
    const lists = app.lists();
    if (lists.length === 0) {
      throw new Error("No reminder lists found");
    }
    return lists[0];
  }
}

function createReminder(options) {
  const app = Application("Reminders");
  const defaults = {
    listName: "Reminders",
    name: "New Reminder",
    body: "",
    minutesFromNow: 5,
    priority: 0 // 0=none, 1=high, 5=medium, 9=low
  };

  const opts = Object.assign({}, defaults, options);

  try {
    const list = getList(app, opts.listName);

    // Calculate reminder time
    const reminderTime = new Date(Date.now() + opts.minutesFromNow * 60 * 1000);

    const reminderProps = {
      name: opts.name,
      remindMeDate: reminderTime
    };

    if (opts.body) {
      reminderProps.body = opts.body;
    }

    if (opts.priority > 0) {
      reminderProps.priority = opts.priority;
    }

    if (opts.dueDate) {
      reminderProps.dueDate = opts.dueDate;
    }

    const reminder = app.Reminder(reminderProps);
    list.reminders.push(reminder);

    return {
      success: true,
      list: list.name(),
      reminderTime: reminderTime.toLocaleString(),
      name: opts.name
    };
  } catch (error) {
    return {
      success: false,
      error: error.message
    };
  }
}

// Example usage - create a reminder for 5 minutes from now
function run() {
  const result = createReminder({
    name: "Call back 555-555-5555",
    body: "Return phone call",
    minutesFromNow: 5,
    priority: 1 // High priority
  });

  if (result.success) {
    return `Reminder "${result.name}" created in '${result.list}' for ${result.reminderTime}`;
  } else {
    return `Error: ${result.error}`;
  }
}
