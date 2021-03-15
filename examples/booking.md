## Booking

Booking

```js
import { AdminClient } from "@nettu/scheduler-sdk";

const address = "https://localhost:5000";
const client = new AdminClient(address);

const user = await client.users.create();
const service = await client.services.create();
const schedule = await client.schedule.create();
await client.services.addUser({
    userId: user.id,
    serviceId: service.id,
    scheduleId: schedule.id
})

const bookingSlots = await client.services.bookingslots({
    date: "2020-10-10",
    timezone: "UTC",
    interval: 1000*60*10,
    duration: 1000*60*30
});
console.log(bookingSlots);

await client.events.insert({
    services: service.id,
    remindAt: 1000*60*30
});

const bookingSlots = await client.services.bookingslots({
    date: "2020-10-10",
    timezone: "UTC",
    interval: 1000*60*10,
    duration: 1000*60*30
});
console.log(bookingSlots);

```