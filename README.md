https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- what needs to be done to make nettu marketplace work with this ? 
    - service module with crud for its general info
    - update users on service
    - get booking slots for service
- protect create account
- account admin routes
  - create user
  - create calendar
- reminders with webhook calls


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


## Defining services, resources and availibility
- booking objects will not be created 
- service is a bookable entity
- users can be registered on the service
- events to listen to. Webhook or kafka or something ? 
- how to handle booking requests ? 

account:
  alloweduseractions

service:
  booking_options
    - duration
  bookingslots_duration:
  allow_more_booking_requests_in_queue_than_resources
  breaks
  users 
  metadata

get_service_bookingslots
  - fetch service by service id
  - get user_ids and corresponding calendar_ids from service object
  - get freebusy from these calendars
  - generate bookingslots from freebusy for every resource


resource:
  userId
  calendars: Calendar[]
  availibility_time
