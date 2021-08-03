insert into accounts (account_uid, secret_api_key, public_jwt_key, settings) values 
	('510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'GDPOE7N7', '1SacOWlj', '{}');
insert into account_integrations (account_uid, client_id, client_secret, redirect_uri, "provider") values 
	('510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'client_id_123', 'client_sec_123', 'test.com', 'google');
insert into account_integrations (account_uid, client_id, client_secret, redirect_uri, "provider") values 
	('510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'client_id_123', 'client_sec_123', 'test.com', 'outlook');
insert into users (user_uid, account_uid, metadata) values 
	('a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}');
insert into user_integrations (user_uid, account_uid, refresh_token, access_token, access_token_expires_ts, "provider") values 
	('a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'ref_123', 'acc_1231', 1232131, 'google');
insert into user_integrations (user_uid, account_uid, refresh_token, access_token, access_token_expires_ts, "provider") values 
	('a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 'ref_123_o', 'acc_1231_o', 1232131, 'outlook');
insert into services (service_uid, account_uid, multi_person, metadata) values 
	('6d469137-9b66-40fe-a314-3c07a1200415', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}', '{}');
insert into service_users (service_uid, user_uid, buffer_before, buffer_after, closest_booking_time) values 
	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 0, 0, 0);
insert into calendars (calendar_uid, user_uid, account_uid, settings, metadata) values 
	('f5444e78-b224-46ef-b75b-deed50790838', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}', '{}');
insert into calendars (calendar_uid, user_uid, account_uid, settings, metadata) values 
	('5688e62f-cee2-466e-b2f2-5b0717558a11', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', '{}', '{}');
insert into calendar_events (
		event_uid,
		calendar_uid, 
		user_uid, 
		account_uid, 
		start_ts,
		duration,
		end_ts,
		busy,
		created,
		updated,
		exdates,
		metadata
	) values 
	(
		'19f3ef7a-3b30-433b-be62-93810694b639',
		'5688e62f-cee2-466e-b2f2-5b0717558a11', 
		'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 
		'510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 
		0,
		0,
		0,
		false,
		0,
		0,
		'{}',
		'{}'
	);
insert into event_reminder_versions (event_uid, "version") values('19f3ef7a-3b30-433b-be62-93810694b639', 0);
insert into reminders (event_uid, account_uid, remind_at, "version", identifier) values('19f3ef7a-3b30-433b-be62-93810694b639', '510ae4e5-3fa5-4ab4-8f87-fe33c2c9205c', 0, 0, 'review');
insert into calendar_event_reminder_generation_jobs (event_uid, "timestamp", "version") values('19f3ef7a-3b30-433b-be62-93810694b639', 0, 0);
-- insert into service_user_busy_calendars (service_uid, user_uid, calendar_uid) values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'f5444e78-b224-46ef-b75b-deed50790838');
-- insert into service_user_busy_calendars (service_uid, user_uid, calendar_uid) values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', '5688e62f-cee2-466e-b2f2-5b0717558a11');
-- insert into service_user_external_busy_calendars (service_uid, user_uid, ext_calendar_id, "provider") values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'busy_1', 'google');
-- insert into service_user_external_busy_calendars (service_uid, user_uid, ext_calendar_id, "provider") values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'busy_2', 'google');
-- insert into service_user_external_busy_calendars (service_uid, user_uid, ext_calendar_id, "provider") values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'busy_1', 'outlook');
-- insert into service_user_external_busy_calendars (service_uid, user_uid, ext_calendar_id, "provider") values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'busy_2', 'outlook');
-- insert into service_user_external_busy_calendars (service_uid, user_uid, ext_calendar_id, "provider") values 
-- 	('6d469137-9b66-40fe-a314-3c07a1200415', 'a6b512cf-c4d8-49ac-9103-c9d8e9453bf2', 'busy_3', 'outlook');



SELECT jsonb_agg((u.*)) AS users FROM services AS s 
LEFT JOIN (
    SELECT su.*, array_agg(c.calendar_uid) AS busy, jsonb_agg(json_build_object('provider', ext_c.provider, 'busy_calendars', ext_c.busy_calendars)) as busy_ext FROM service_users AS su 
    LEFT JOIN service_user_busy_calendars as c
    ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
    LEFT JOIN service_user_external_busy_calendars as ext_c
    ON su.service_uid = ext_c.service_uid AND su.user_uid = ext_c.user_uid
    GROUP BY su.service_uid, su.user_uid
) as u
ON u.service_uid = s.service_uid 
GROUP BY s.service_uid;


SELECT jsonb_agg((u.*)) AS users FROM services AS s 
LEFT JOIN (
    SELECT su.*, array_agg(c.calendar_uid) AS busy, jsonb_agg(json_build_object('provider', ext_c.provider, 'busy_calendars', ext_c.busy_calendars)) as busy_ext FROM service_users AS su 
    LEFT JOIN service_user_busy_calendars as c
    ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
    LEFT JOIN service_user_external_busy_calendars as ext_c
    ON su.service_uid = ext_c.service_uid AND su.user_uid = ext_c.user_uid
    GROUP BY su.service_uid, su.user_uid
) as u
ON u.service_uid = s.service_uid 
GROUP BY s.service_uid;

SELECT jsonb_agg((u.*)) AS users FROM services AS s 
LEFT JOIN (
    SELECT su.*, array_agg(c.calendar_uid) AS busy, jsonb_agg(json_build_object('provider', ext_c.provider, 'busy_calendars', ext_c.busy_calendars)) as busy_ext FROM service_users AS su 
    LEFT JOIN service_user_busy_calendars as c
    ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
    LEFT JOIN service_user_external_busy_calendars as ext_c
    ON su.service_uid = ext_c.service_uid AND su.user_uid = ext_c.user_uid
    GROUP BY su.service_uid, su.user_uid, busy_ext #>> '{provider}'
) as u
ON u.service_uid = s.service_uid 
GROUP BY s.service_uid;

CREATE VIEW service_users_info AS
	SELECT su.*, array_agg(c.calendar_uid) AS busy, jsonb_agg(json_build_object('provider', ext_c.provider, 'busy_calendars', ext_c.busy_calendars)) as busy_ext FROM service_users AS su 
	LEFT JOIN service_user_busy_calendars as c
	ON su.service_uid = c.service_uid AND su.user_uid = c.user_uid
	LEFT JOIN service_user_external_busy_calendars as ext_c
	ON su.service_uid = ext_c.service_uid AND su.user_uid = ext_c.user_uid
	GROUP BY su.service_uid, su.user_uid;


SELECT s.*, jsonb_agg((u.*)) AS users FROM services AS s 
LEFT JOIN service_users_info AS u
ON u.service_uid = s.service_uid 
GROUP BY s.service_uid;