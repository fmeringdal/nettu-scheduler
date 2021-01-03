https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- what needs to be done to make nettu marketplace work with this ? 
    - get service by id
    - list services for account
    - account admin routes 
      - get user with calendar list
      - create user
      - create calendar
- use interval for bookingslots request
- protect create account
- reminders with webhook calls
- account with alloweduseractions


## Backlog

- smarter mongodb schema
- More api tests for [calendarevent, booking]
- error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
- frontend for booking


## Need to have a data model that will support google and outlook calendars in the future
- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ? 
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids

