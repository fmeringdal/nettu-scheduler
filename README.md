<div align="center">
<img width="400" src="docs/logo.png" alt="logo">
</div>

# Nettu scheduler
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build status](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)
[![codecov](https://codecov.io/gh/fmeringdal/nettu-scheduler/branch/master/graph/badge.svg?token=l5z2mzzdHu)](https://codecov.io/gh/fmeringdal/nettu-scheduler)

## Overview

`Nettu scheduler` is a self-hosted calendar and scheduler server that aims to provide the building blocks for building calendar / booking apps with ease. It has a simple REST API and also a [JavaScript SDK](https://www.npmjs.com/package/@nettu/sdk-scheduler) and [Rust SDK](https://crates.io/crates/nettu_scheduler_sdk). 

It supports authentication through api keys for server - server communication and JSON Web Tokens for browser - server communication.

## Features
- **Booking**: Create a `Service` and register `User`s on it to make them bookable.
- **Calendar Events**: Supports recurrence rules, flexible querying and reminders.
- **Calendars**: For grouping `Calendar Event`s.
- **Freebusy**: Find out when `User`s are free and when they are busy.
- **Metadata queries**: Add key-value metadata to your resources and then query on that metadata 
- **Webhooks**: Notifying your server about `Calendar Event` reminders.

<br/>

<div align="center">
<img src="docs/flow.svg" alt="Application flow">
</div>


## Table of contents

  * [Quick start](#quick-start)
  * [Examples](#examples)
  * [Contributing](#contributing)
  * [License](#license)
  * [Special thanks](#special-thanks)


## Quick start

First of all we will need a running instance of the server. The quickest way to start one
is with `docker`:
```bash
docker run -p 5000:5000 fmeringdal/nettu-scheduler:latest
```
or if you want to build it yourself with `cargo`:
```bash
cd scheduler
cargo run inmemory
```
Both of these methods will start the server with an inmemory data store which should never
be used in production, but is good enough while just playing around.
For information about setting up this server for deployment, read [here](./docs/deployment.md).

Now when we have the server running we will need an `Account`. To create an `Account`
we will need the `CREATE_ACCOUNT_SECRET_CODE` which you will find in the server logs
during startup (it can also be set as an environment variable).
```bash
curl -X POST -H "Content-Type: application/json" -d '{"code": "REPLACE_ME"}' -v http://localhost:5000/api/v1/account
```
The previous command will create an `Account` and the associated `secretApiKey` which is all you need when
your application is going to communicate with the Nettu Scheduler server.

Quick example of how to create and query a user
```bash
export SECRET_API_KEY="REPLACE_ME"

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