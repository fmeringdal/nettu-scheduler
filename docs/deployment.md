## Deployment of Nettu Scheduler Server

Use the docker image: `fmeringdal/nettu-scheduler`.

Or build from source: 
```bash
cd scheduler
cargo run --release
```

Then setup a mongodb with the indexes specified [here](./mongodb.md).
Lastly provide the following environment variables to the `nettu scheduler` server:
```bash
# The connection string to the database
MONGODB_CONNECTION_STRING
# The mongo database name that the server should use
MONGODB_NAME
```

