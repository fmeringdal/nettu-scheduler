## Json Web Token 

If you want your end-users to interact with the nettu scheduler API you can give them
a Json Web Token signed with your private RSA key.

:w

This is what your code might look like on your server side

```js
import jwt from "jsonwebtoken"; 
import { AdminClient, Permissions } from "@nettu/scheduler-sdk";

const address = "https://localhost:5000";
const token = "YOUR_TOKEN";
const client = new AdminClient(address);
const user = await client.users.create();

await client.account.setPublicJWTKey("YOUR_PUBLIC_JWT_KEY");


const handleJWTRequest = async(user) => {
    if(!user.schedulerUserId) {
        const { id } = await client.user.create();
        user.schedulerUserId = id;
    }

    const token = jwt.sign({
        schedulerUserId: user.schedulerUserId,
        policy: {
            allow: "*",
            deny: "DeleteCalendar"
        }
    }, {
        alg: "RS256",
        key: "YOUR_PRIVATE_KEY"
    });
    

    res.send(token);
}

```


This is what your code might look like in the frontend
```js
import { UserClient} from "@nettu/scheduler-sdk";

const address = "https://localhost:5000";

const getJWT = async(): string => {
    // HERE GOES LOGIC FOR CALLING YOUR OWN SERVERS ENDPOINT WHICH RETURNS A JWT
}

const token = await getJWT();
const client = new UserClient(address, token);

const event = await client.events.update({ eventId, weekly });

```