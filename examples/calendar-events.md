## Calendar Events

`Calendar Event`s are events in a `Calendar` and can be either a single event or a recurring event. It is also possible to store `Metadata` on the `Calendar Event` to add additional fields if desired (e.g. title, description, videolink, etc.), and also query on those fields.


```js
import { NettuClient, config } from "@nettu/scheduler-sdk";

config.baseUrl = "https://localhost:5000";
const client = new NettuClient({ apiKey: "REPLACE_ME" });

// Create a User
const { user } = await client.users.create();
// Create the Calendar theat the CalendarEvent will belong to
const { calendar } = await client.calendars.create(user.id, {
    // Starts on monday
    weekStart: 0,
    // Timezone for the calendar
    timezone: "UTC"
});
// Create a CalendarEvent that repeats daily
const { event } = await client.events.insert({ 
    userId, 
    calendarId, 
    startTs: 0,
    duration: 1000*60*30, // 30 minutes in millis
    recurrence: {
        frequenzy: "daily"
    },
    metadata: {
        mykey: "myvalue"
    }
});

// Retrieve event instances in a given Timespan
const { instances } = await client.events.getInstances(event.id, {
    startTs: 0,
    endTs: 1000*60*60*24
});

// Retrieve CalendarEvents by metadata
const skip = 0;
const limit = 100;
const { events } = await client.events.findByMeta({
    key: "mykey",
    value: "myvalue"
}, skip, limit);

```