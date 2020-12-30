https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- add event_id on eventInstance
- exdates usecases and tests
- use the different results that are unused
- better error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
- https://developer.makeplans.net/#services


## backlog

- smarter mongodb schema
- More api tests for [calendarevent, booking]
- think about how to do auth (nettu ee will likely also use this and maybe nettmeet)
  - should it be just microservice for nettu to start with ? (public / private cert jwt, check google)
  - same as nettu meeting with external api calling endpoints ?
  - both
