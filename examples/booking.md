## Booking

`Nettu scheduler` provides a `Service` type that makes it really easy to build a booking application. The `Service` type represents a bookable service that your app provides.

It works by adding the `User`s to the `Service` that is going to be bookable. Each `User`
on the `Service` can have a bunch of different settings for when they can be booked (availibility schedule, closest booking time, buffers etc.).
Then you can then easily query the `Service` for the available bookingslots. When
a booking is made you can represent it as a `CalendarEvent` with the booked `User`
as the owner of the `CalendarEvent` and the `User` will no longer be bookable
during that timeperiod.

An important point is to not store the booking resource itself in `Nettu scheduler`, but in your own application as your application is the one that contains all the information about the participants and metadata of the booking. It is reccomended that `Nettu scheduler` is just going to be used for calculating bookingslots, and not to try to fit your booking data model into the `CalendarEvent` resource type (unnless it is a really simple one). 


```js
import { NettuClient, config } from "@nettu/scheduler-sdk";

config.baseUrl = "https://localhost:5000";
const client = new NettuClient({ apiKey: "REPLACE_ME" });

// Create a Service
const { service } = await client.service.create();
// Create a User
const { user } = await client.user.create();
// Create a Schedule for the User
const { schedule } = await client.schedule.create(user.id, { timezone: "Europe/Oslo" });
// Register the User on the Service with the specified Schedule as availibility and
// also a buffer time after every service event 
await client.service.addUser({
    userId: user.id,
    serviceId: service.id,
    scheduleId: schedule.id,
    buffer: 10
});

// Now query for the available bookingslots
const bookingSlots = await client.service.bookingslots({
    date: "2020-10-10",
    timezone: "UTC",
    interval: 1000*60*10,
    duration: 1000*60*30
});
console.log(bookingSlots);

// Insert a CalendarEvent that represents the booking selected
// by the end user, the User will no longer be bookable in this timeperiod
await client.events.create(user.id, {
    startTs: bookingSlots[0].startTs,
    duration: 1000*60*30, // 30 minutes in millis
    isService: true, // Flagging this event as a service event so that possible service buffers will be created correctly
    busy: true, // The user will be busy during this time and not bookable
    // Optional if you want to receive a webhook notification 15 minutes before
    // the booking
    reminder: {
        minutesBefore: 15
    }
});
```