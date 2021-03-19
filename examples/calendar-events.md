## Calendar Events

`Calendar Event`s are events in a `Calendar` and can be either a single event or a recurring event. It is also possible to store `Metadata` on the `Calendar Event` to add additional fields if desired (e.g. title, description, videolink, etc.), and also query on those fields.


```js
import { NettuClient, Frequenzy, config } from "@nettu/scheduler-sdk";

config.baseUrl = "http://localhost:5000/api/v1";
const client = NettuClient({ apiKey: "YOUR_API_KEY" });

// Create a User
const userRes = await client.user.create();
const { user } = userRes.data!;

// Create the Calendar theat the CalendarEvent will belong to
const calendarRes = await client.calendar.create(user.id, {
    // Starts on monday
    weekStart: 0,
    // Timezone for the calendar
    timezone: "UTC"
});
const { calendar } = calendarRes.data!;

// Create a CalendarEvent that repeats daily
const eventRes = await client.events.create(user.id, {
    calendarId: calendar.id,
    startTs: 0,
    duration: 1000 * 60 * 30, // 30 minutes in millis
    recurrence: {
        freq: Frequenzy.Daily,
        interval: 1
    },
    metadata: {
        mykey: "myvalue"
    }
});
const { event } = eventRes.data!;

// Retrieve event instances in a given Timespan
const instancesRes = await client.events.getInstances(event.id, {
    startTs: 0, // unix timestamp 0 -> 1970.1.1
    endTs: 1000 * 60 * 60 * 24
});

const { instances } = instancesRes.data!;
console.log(instances);

// Retrieve CalendarEvents by metadata
const skip = 0;
const limit = 100;
const eventMetaQuery = await client.events.findByMeta({
    key: "mykey",
    value: "myvalue"
}, skip, limit);

const { events } = eventMetaQuery.data!;
console.log(events);

```