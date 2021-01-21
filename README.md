https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos


## Backlog

- Webhook verification challenge ? or just have email validation on account
- Delete user cleanup actions
- Account with alloweduseractions ? maybe add it to jwt
- More api tests for [calendarevent, booking]
- smarter mongodb schema
- is more account admin routes needed?
- protect create account with real emails
- frontend for booking

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
