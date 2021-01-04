https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- Maybe repos dont need to be arc ? Maybe just use new method for collection ??
- take another look at the way domain objects are ser- and deserialized into mongodb
- remove option for end_ts and set max timestamp
- setup the skeleton for rust integration tests (mongodb docker compose without volume)
- protect create account
- reminders with webhook calls
- account with alloweduseractions
- delete user cleanup actions

## Backlog

- smarter mongodb schema
- is more account admin routes needed?
- More api tests for [calendarevent, booking]
- error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
- frontend for booking

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
