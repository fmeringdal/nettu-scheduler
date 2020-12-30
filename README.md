https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- think about how to do auth (Nettu EE will likely also use this and maybe nettmeet)
  - support public key / private key verification with public key from company
    - need a company repo
    - how to fetch company from req ? Header value
  - support api secret key (need a user repo)
- https://developer.makeplans.net/#services
- frontend for booking
  - admin portal for external application
  - callback for connecting to google calendar and outlook calendar
  - calendar page? Neh


## backlog

- smarter mongodb schema
- More api tests for [calendarevent, booking]
- error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
