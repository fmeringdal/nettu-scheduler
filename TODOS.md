
## Todos

- fjerne context fra UseCase
- server -> scheduler
- More admin and user controllers for different usecases
- calendar settings, default and on creation
https://rust-lang.github.io/api-guidelines/type-safety.html
https://github.com/Qovery/engine
https://vector.dev/
- Better telemtry: implement Display for usecase: https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/#5-1-the-tracing-crate

## Backlog

- Shared calendars, events etc
- metadata
- More api tests for [calendarevent, booking]
- smarter mongodb schema

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
