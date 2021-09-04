<div align="center">
<img width="400" src="docs/logo.png" alt="logo">
</div>

# Nettu scheduler
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/release.yml/badge.svg)](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/fmeringdal/nettu-scheduler/branch/master/graph/badge.svg?token=l5z2mzzdHu)](https://codecov.io/gh/fmeringdal/nettu-scheduler)

## Overview

`Nettu scheduler` is a self-hosted calendar and scheduler server that aims to provide the building blocks for building calendar / booking apps with ease. It has a simple REST API and also a [JavaScript SDK](https://www.npmjs.com/package/@nettu/sdk-scheduler) and [Rust SDK](https://crates.io/crates/nettu_scheduler_sdk). 

It supports authentication through api keys for server - server communication and JSON Web Tokens for browser - server communication.

## Features
- **Booking**: Create a `Service` and register `User`s on it to make them bookable.
- **Calendar Events**: Supports recurrence rules, flexible querying and reminders.
- **Calendars**: For grouping `Calendar Event`s.
- **Freebusy**: Find out when `User`s are free and when they are busy.
- **Integrations**: Connect your Nettu, Google and Outlook calendars
- **Multi-tenancy**: All resources are grouped by `Account`s. 
- **Metadata queries**: Add key-value metadata to your resources and then query on that metadata 
- **Webhooks**: Notifying your server about `Calendar Event` reminders.

<br/>

<div align="center">
<img src="docs/flow.svg" alt="Application flow">
</div>

## Quick start

The server is using PostgreSQL for persistence, so we will need to spin up that first: 
```bash
cd scheduler
docker-compose -f integrations/docker-compose.yml up -d
```

Now we are ready to start the `nettu-scheduler` server with `docker`
```bash
docker run -p 5000:5000 -e ACCOUNT_API_KEY="REPLACE_ME" \ 
    -e DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nettuscheduler" \ 
    fmeringdal/nettu-scheduler:latest 
```
or if you want to build it yourself with `cargo`:
```bash
cd scheduler
export ACCOUNT_API_KEY="REPLACE_ME"
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/nettuscheduler"
cargo run
```

The `ACCOUNT_API_KEY` environment variable is going to create an `Account` (if it does not already exist) during
server startup with the given key. `Account`s act as tenants in the server, and it is possible to create multiple `Account`s by using the `CREATE_ACCOUNT_SECRET_CODE` which you can provide as an environment variable.

Quick example of how to create and query a user
```bash
export SECRET_API_KEY="REPLACE ME WITH YOUR API KEY"

# Create a user with metadata
curl -X POST -H "Content-Type: application/json" -H "x-api-key: $SECRET_API_KEY" -d '{"metadata": { "groupId": "123" }}' http://localhost:5000/api/v1/user

# Get users by metadata
curl -H "x-api-key: $SECRET_API_KEY" "http://localhost:5000/api/v1/user/meta?key=groupId&value=123"
```

Please see below for links to more examples.


## Examples

* [Calendars and Events](examples/calendar-events.md)

* [Booking](examples/booking.md)

* [Reminders](examples/reminders.md)

* [Creating JWT for end-users](examples/jwt.md)


## Contributing

Contributions are welcome and are greatly appreciated!

## License

[MIT](LICENSE) 

## Special thanks

* [Lemmy](https://github.com/LemmyNet/lemmy) for inspiration on how to use cargo workspace to organize a web app in rust. 
* [The author of this blog post](https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/) for an excellent introduction on how to do telemetry in rust. 
