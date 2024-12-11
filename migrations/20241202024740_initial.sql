-- Add migration script here
-- this file is used for postgresql database initialization
-- create user table
CREATE TABLE IF NOT EXISTS users (
    -- bigserial is also i64 size
    id bigserial PRIMARY KEY,
    fullname varchar(64) NOT NULL,
    email varchar(64) NOT NULL,
    -- hashed argon2 password, length is 97
    password_hash varchar(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');


-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id serial PRIMARY KEY,
    name varchar(64),     -- if it is single chat, chat name can be null
    type chat_type NOT NULL,
    -- user id list
    members bigint[] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL REFERENCES chats(id),
    sender_id bigint NOT NULL REFERENCES users(id),
    content text NOT NULL,
    images text[],
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and create_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);