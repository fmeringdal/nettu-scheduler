-- insert into accounts (account_uid, secret_api_key, public_jwt_key, settings) values ('510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'GDPOE7N7', '1SacOWlj', '{}');
-- insert into users (user_uid, account_uid, metadata) values ('a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}');
-- insert into users (user_uid, account_uid, metadata) values ('7b89f555-f728-409d-b06a-896063b38462', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}');
-- insert into services (service_uid, account_uid, metadata) values ('6d469137-9b66-40fe-a314-3c07a1200415', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}');
-- insert into service_users (service_uid, user_uid, "buffer", closest_booking_time) values ('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 0, 0);
-- insert into service_users (service_uid, user_uid, "buffer", closest_booking_time) values ('6d469137-9b66-40fe-a314-3c07a1200415', '7b89f555-f728-409d-b06a-896063b38462', 0, 0);
-- insert into calendars (calendar_uid, user_uid, account_uid, settings, metadata) values ('f5444e78-b224-46ef-b75b-deed50790838', '7b89f555-f728-409d-b06a-896063b38462', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}', '{}');
-- insert into service_user_busy_calendars (service_uid, user_uid, calendar_uid) values ('6d469137-9b66-40fe-a314-3c07a1200415', '7b89f555-f728-409d-b06a-896063b38462', 'f5444e78-b224-46ef-b75b-deed50790838');

-- ALTER TABLE service_users 
-- DROP COLUMN "buffer";
-- ALTER TABLE service_users
-- ADD COLUMN buffer_after BIGINT NOT NULL;
-- ALTER TABLE service_users
-- ADD COLUMN buffer_before BIGINT NOT NULL;
-- ALTER TABLE users
-- ADD COLUMN integrations JSON;
-- ALTER TABLE calendars
-- ADD COLUMN synced JSON;
-- ALTER TABLE calendar_events
-- ADD COLUMN synced_events JSON;
ALTER TABLE service_users
ADD COLUMN google_busy_calendars text[] NOT NULL;