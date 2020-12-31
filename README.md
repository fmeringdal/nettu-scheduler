https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- think about how to do auth (Nettu EE will likely also use this and maybe nettmeet)
  - create user if user does not exist
  - commong crud for mongodb, dont unwrap objectid
  - user should have store the company name and an external id
  - more test for public key / private key verification with public key from company
    - usecases for uploading public private key
  - support api secret key
- company -> account
- Look more into this: https://developer.makeplans.net/#services
- use strings for rruleoptions.frequenzy
- frontend for booking
  - admin portal for external application
  - callback for connecting to google calendar and outlook calendar
  - company page or calendar page?


## backlog

- smarter mongodb schema
- More api tests for [calendarevent, booking]
- error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
