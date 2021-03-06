
## Todos

- metadata
- Path naming for admin and user routes 
  - Admin routes maybe just need an additional userid in path params
  - Complete SDK
- sjekke ut postgresql
https://rust-lang.github.io/api-guidelines/type-safety.html
https://github.com/Qovery/engine
https://vector.dev/
- Better telemtry: implement Display for usecase: https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/#5-1-the-tracing-crate

## Backlog

- Shared calendars, events etc
- smarter mongodb schema
- more recurrence validation 

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
