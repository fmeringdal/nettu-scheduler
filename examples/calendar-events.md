## Calendar Events

Calendars 

```js
import { AdminClient } from "@nettu/scheduler-sdk";

const address = "http://localhost:5000";
const token = "YOUR_TOKEN";
const client = new AdminClient(address);
const user = await client.users.create();

const calendar = await client.calendars.create();
const event = await client.events.insert({ userId, calendarId, daily });

```