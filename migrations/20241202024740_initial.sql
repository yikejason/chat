-- Add migration script here
-- this file is used for postgresql database initialization
-- create user table
CREATE TABLE IF NOT EXISTS users (
    -- bigserial is also i64 size
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL,
    fullname varchar(64) NOT NULL,
    email varchar(64) NOT NULL,
    -- hashed argon2 password, length is 97
    password_hash varchar(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces(
    id bigserial PRIMARY KEY,
    name varchar(32) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

BEGIN;
    INSERT INTO users(id, ws_id, fullname, email, password_hash)
    VALUES (0, 0, 'super user', 'super@none.org', '');
    INSERT INTO workspaces(id, name, owner_id)
    VALUES (0, 'none', 0);
COMMIT;

-- add foreign key constraint for ws_id for users
ALTER TABLE users
    ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');


-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id bigserial PRIMARY KEY, -- bigserial is also i64 size （int8）  serial is int4
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    name varchar(64),     -- if it is single chat, chat name can be null
    type chat_type NOT NULL,
    members bigint[] NOT NULL DEFAULT '{}',  -- user id list
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (ws_id, name)
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL REFERENCES chats(id),
    sender_id bigint NOT NULL REFERENCES users(id),
    content text NOT NULL,
    files text[] DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and create_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);

-- create index for chat members    GIN represents 复杂的 index search  eg: array json etc
CREATE INDEX IF NOT EXISTS chat_members_index ON chats USING GIN(members);
