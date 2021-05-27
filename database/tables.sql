-- https://wiki.postgresql.org/wiki/Referential_Integrity_Tutorial_%26_Hacking_the_Referential_Integrity_tables

CREATE TABLE accounts (
    account_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    secret_api_key varchar(255) NOT NULL UNIQUE,
    public_jwt_key varchar(255),
);

-- https://www.youtube.com/watch?v=NfLRqTTbkkU

-- METADATA
-- https://dba.stackexchange.com/questions/115825/jsonb-with-indexing-vs-hstore
-- how to index ? Try to make table work, otherwise jsonb
-- same table for all metadata or own metadata table for each resource
-- https://stackoverflow.com/questions/22372660/is-it-possible-to-have-an-dynamic-foreign-key-and-what-is-the-best-correct-to-d
-- https://stackoverflow.com/questions/13311188/what-is-the-best-design-for-a-database-table-that-can-be-owned-by-two-different/13317463#13317463

-- CREATE TABLE metadata {
--     id uuid primary key DEFAULT uuid_generate_v4() NOT NULL,
--     "resource" varchar(),
--     "key" varchar(255) NOT NULL,
--     "value" varchar(255) NOT NULL,
-- }
CREATE TABLE metadata {
    id INTEGER PRIMARY KEY,
    "key" varchar(255) NOT NULL,
    "value" varchar(255) NOT NULL,
    user_id uuid,
    CHECK (
        (group_id IS NOT NULL AND user_id IS NULL)
        OR (group_id IS NULL AND user_id IS NOT NULL)
    )
}

CREATE TABLE metadata_users {
    id INTEGER PRIMARY KEY,
    user_id uuid,
    "key" varchar(255) NOT NULL,
    "value" varchar(255) NOT NULL,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id) 
        REFERENCES users(id),
}