https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- take another look at the way domain objects are ser- and deserialized into mongodb
- More api tests for [calendarevent, booking]
- reminders with webhook calls
- account with alloweduseractions
- delete user cleanup actions

## Backlog

- smarter mongodb schema
- is more account admin routes needed?
- protect create account with real emails
- error handling: 
    1: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
    2: https://theomn.com/rust-error-handling-for-pythonistas/
- frontend for booking

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
