## Setup of MongoDB

### Indexes
```bash
db.accounts.createIndex({ "attributes.key": 1, "attributes.value": 1 })
db.calendars.createIndex({ "user_id": 1 })
db.calendars.createIndex({ "metadata.key": 1, "metadata.value": 1 })
db.calendar-events.createIndex({ "calendar_id": 1, "start_ts": 1, "end_ts": -1 })
db.calendar-events.createIndex({ "user_id": 1 })
db.calendar-events.createIndex({ "metadata.key": 1, "metadata.value": 1 })
db.calendar-event-reminder-expansion-jobs.createIndex({ "timestamp": 1 })
db.calendar-event-reminder-expansion-jobs.createIndex({ "event_id": 1 })
db.calendar-event-reminders.createIndex({ "remind_at": 1 })
db.calendar-event-reminders.createIndex({ "event_id": 1 })
db.schedules.createIndex({ "user_id": 1 })
db.services.createIndex({ "ids": 1 })
db.services.createIndex({ "metadata.key": 1, "metadata.value": 1 })
db.users.createIndex({ "metadata.key": 1, "metadata.value": 1 })
```