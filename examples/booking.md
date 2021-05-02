## Booking

`Nettu scheduler` provides a `Service` type that makes it really easy to build a booking application. The `Service` type represents a bookable service that your app provides.

It works by adding `User`s to the `Service` that are going to be bookable. Each `User`
on the `Service` can have a bunch of different settings for when they can be booked (availibility schedule, closest booking time, buffers etc.).
Then you can then easily query the `Service` for the available bookingslots. When
a booking is made you can represent it as a `CalendarEvent` with the booked `User`
as the owner of the `CalendarEvent` and the `User` will no longer be bookable
during that timeperiod.

An important point is to not store the booking resource itself in `Nettu scheduler`, but in your own application as your application is the one that contains all the information about the participants and metadata of the booking. It is reccomended that `Nettu scheduler` is just going to be used for calculating bookingslots, and not to try to fit your booking data model into the `CalendarEvent` resource type (unnless it is a really simple one). 


```js
import { NettuClient } from "@nettu/scheduler-sdk";

const client = NettuClient({ apiKey: "YOUR_API_KEY" });

// Create a Service
const serviceRes = await client.service.create();
const { service } = serviceRes.data!;

// Create a User
const userRes = await client.user.create();
const { user } = userRes.data!;

// Create a Schedule for the User
const scheduleRes = await client.schedule.create(user.id, { timezone: "Europe/Oslo" });
const { schedule } = scheduleRes.data!;

// Calendar that will be used to store bookings
const calendarRes = await client.calendar.create(user.id, {
    timezone: "Europe/Oslo",
    weekStart: 0
});
const { calendar } = calendarRes.data!;

// Register the User on the Service with the specified Schedule as availibility and
// also a buffer time after every service event 
await client.service.addUser(service.id, {
    userId: user.id,
    availibility: {
        variant: "Schedule",
        id: schedule.id
    },
    // Calendars that should be used to calculate busy time
    busy: [calendar.id],
    // Make User unbookable for 10 minutes after a booking 
    buffer: 10
});

// Now query for the available bookingslots
const bookingSlotsRes = await client.service.getBookingslots(service.id, {
    startDate: "2030-10-10",
    endDate: "2030-10-10",
    ianaTz: "Europe/Oslo",
    interval: 1000 * 60 * 10,
    duration: 1000 * 60 * 30
});
const bookingSlotsBefore = bookingSlotsRes.data!.dates[0].slots;

// Insert a CalendarEvent that represents the booking selected
// by the end user, the User will no longer be bookable in this timeperiod
await client.events.create(user.id, {
    startTs: bookingSlotsBefore[0].start,
    calendarId: calendar.id,
    duration: 1000 * 60 * 30, // 30 minutes in millis
    isService: true, // Flagging this event as a service event so that possible service buffers will be created correctly
    busy: true, // The user will be busy during this time and not bookable
    // Optional if you want to receive a webhook notification 15 minutes before
    // the booking
    reminder: {
        minutesBefore: 15
    }
});

const bookingSlotsRes2 = await client.service.getBookingslots(service.id, {
    startDate: "2030-10-10",
    endDate: "2030-10-10",
    ianaTz: "Europe/Oslo",
    interval: 1000 * 60 * 10,
    duration: 1000 * 60 * 15
});
const bookingSlotsAfter = bookingSlotsRes2.data!.dates[0].slots;

// See that user is no longer available during the booked event + buffer time 
console.log({
    before: bookingSlotsBefore.length,
    after: bookingSlotsAfter.length
})
```
