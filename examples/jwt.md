## Json Web Token 

Json Web Tokens are useful if you want your end-users to interact with the nettu scheduler API from the browser. Just upload your public RSA key (only RS256 algorithm is supported) to the `nettu scheduler` and then create jwt by signing the data with your private RSA key.

This is how your server side code might look like

```js
import jwt from "jsonwebtoken"; 
import { NettuClient, config, Permissions } from "@nettu/scheduler-sdk";

config.baseUrl = "https://localhost:5000";
const client = new NettuClient({ apiKey: "REPLACE_ME" });

// Upload your public rsa signing key
await client.account.setPublicJWTKey("YOUR_PUBLIC_KEY");

// A handler in your server that generates JWT for authenticated
// users 
const handleJWTRequest = async(user) => {
    // Check if your user is already associated with a
    // nettu user. Create one if not.
    if(!user.schedulerUserId) {
        const { user: nettuUser } = await client.user.create();
        user.schedulerUserId = nettuUser.id;
    }

    const token = jwt.sign({
        // The nettu scheduler user id (the subject for this token)
        schedulerUserId: user.schedulerUserId,
        // Policy (a.k.a claims)
        policy: {
            allow: "*",
            deny: "DeleteCalendar"
        }
    }, {
        alg: "RS256",
        // Update this to your private key
        key: "YOUR_PRIVATE_KEY"
    });
    
    const { account } = await client.account.me();

    // Return the token back to the frontend
    return {
        token,
        accountId: account.id
    };
}

```


This is what your frontend code might look like 
```js
import { NettuUserClient } from "@nettu/scheduler-sdk";

const getJWT = async() => {
    // HERE GOES LOGIC FOR CALLING YOUR SERVER ENDPOINT WHICH RETURNS A JWT AND ACCOUNT ID
}

const { token, accountId } = await getJWT();
config.baseUrl = "https://localhost:5000";
// Construct the nettu user client
const client = new NettuUserClient({
    token,
    nettuAccount: accountId
});

// Create a Calendar
const { calendar } = await client.calendar.update({
    weekStart: 0,
    timezone: "UTC"
});

```