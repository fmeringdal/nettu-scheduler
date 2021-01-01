https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- what needs to be done to make nettu marketplace work with this ? 
    - define calendar that should be used for defining availibility
- protect create account
- create user
- Look more into this: https://developer.makeplans.net/#services
- frontend for booking
  - admin portal for external application
  - callback for connecting to google calendar and outlook calendar
  - company page or calendar page?


## Backlog

- smarter mongodb schema
- More api tests for [calendarevent, booking]
- reminders
- error handling: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/


## Need to have a data model that will support google and outlook calendars in the future
- oauth2.0 flow with redirect to our frontend customized with account logo


## Defining services, resources and availibility
- service is a bookable entity
- resource is a user registered on the service
- where are bookings created ? Webhook or kafka or something ? 
- when a user is connected to a service they will get a booking calendar assigned to them if not already exists
  where all accepted bookings for them will be created
- how to handle booking requests ? data model in nettu marketplace (because of users), how to get booking times etc ? Also read makeplans

account:
  alloweduseractions

service:
  booking_options
    - duration
  bookingslots_duration:
  allow_more_booking_requests_in_queue_than_resources
  breaks
  resources ? 
  metadata

get_service_bookingslots

resource:
  serviceId
  userId
  availibility_calendar, availibility_time
