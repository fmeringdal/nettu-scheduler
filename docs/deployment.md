## Deployment of Nettu Scheduler Server

Use the docker image: `fmeringdal/nettu-scheduler`.

Or build from source: 
```bash
cd scheduler
cargo run --release
```

Then setup a postgres db with the init script specified [here](../scheduler/crates/infra/migrations/dbinit.sql).
Lastly provide the following environment variables to the `nettu scheduler` server:
```bash
# The connection string to the database
DATABASE_URL
```

