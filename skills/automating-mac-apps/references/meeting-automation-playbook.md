# Meeting Automation Playbook (cross-skill)

End-to-end steps covered by commands/agents:

1) **Setup (`meeting-setup`)**
   - Parse attendees → upsert missing Contacts.
   - Create prep reminders (deck, proposal, metrics) in Reminders with due dates before start.
   - Update/create Calendar event; add alerts (1d/1h/15m).

2) **Start (`meeting-start`)**
   - Detect in-progress meeting from Calendar by current time.
   - Start Voice Memos recording named after the meeting.

3) **End (`meeting-end`)**
   - Stop Voice Memos recording.
   - Pull transcript (if available) from Voice Memos.
   - Draft notes (agenda, decisions, action items) and a follow-up email to attendees.
   - Create Reminders for action items.

Skill touchpoints:
- **Contacts**: attendee upsert/lookup.
- **Reminders**: prep tasks, follow-up tasks.
- **Calendar**: event lookup/create, multi-alerts.
- **Voice Memos**: start/stop recording, transcript scrape/save.
- **Mail**: draft follow-up email.
- **Notes** (forthcoming skill): file meeting notes under `meetings/<company>/<date>-<meeting-title>` and people dossiers under `people/<first>-<last>/...`.
- **mac-apps**: permissions (FDA/Accessibility), ObjC/SQLite helpers.
