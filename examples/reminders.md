## Reminders

Your server can receive reminders before calendar events in the form of webhooks.
This means that `nettu scheduler` will not notify your users through other means like email,
phone etc. That is supposed to be done by your server (if needed) which owns the complete user resources.

```js
import { NettuClient } from "@nettu/scheduler-sdk";

config.baseUrl = "https://localhost:5000";
const client = new NettuClient({ apiKey: "REPLACE_ME" });

const { account } = await client.account.setWebhook("YOUR_URL");
// A generated key used for verifying the webhook the request
const key = account.settings.webhook.key;


const { user } = await client.user.create();
const { event } = await client.events.insert({ 
    userId, 
    calendarId, 
    startTs: 0,
    duration: 1000*60*30, // 30 minutes in millis
    recurrence: {
       frequenzy: "daily" 
    }, 
    reminder: {
        minutesBefore: 20
    } 
});

const webhookReceiverController = (req) => {
    if(req.headers["nettu-scheduler-webhook-key"] !== key) return;

    // Handle reminder by sending email to participants or whatever is needed
}

```