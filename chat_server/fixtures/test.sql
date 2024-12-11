-- insert workspace
INSERT INTO workspaces (name, owner_id)
VALUES ('acme', 0),( 'foo', 0),( 'bar', 0);

-- insert user   all with hashed password '123456'
INSERT INTO users (ws_id, email, fullname, password_hash)
VALUES (1, 'yu@acme.org', 'Yu Tian',
'$argon2id$v=19$m=19456,t=2,p=1$JSU1hRKciCjU+IiDgEdxEw$o3Q2TeHrpgoUslkZXITVeb5zKwUUtFXjeGWmUJd+a7k'),
(1, 'alice@acme.org', 'Alice',
'$argon2id$v=19$m=19456,t=2,p=1$JSU1hRKciCjU+IiDgEdxEw$o3Q2TeHrpgoUslkZXITVeb5zKwUUtFXjeGWmUJd+a7k'),
(1, 'bob@acme.org', 'Bob',
'$argon2id$v=19$m=19456,t=2,p=1$JSU1hRKciCjU+IiDgEdxEw$o3Q2TeHrpgoUslkZXITVeb5zKwUUtFXjeGWmUJd+a7k');

-- insert chats
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members)
VALUES (1, 'general', 'public_channel', '{1,2,3}'),
(1, 'private', 'private_channel', '{1,2,3}');

-- insert unnamed chat
INSERT INTO chats (ws_id, type, members)
VALUES (1, 'single', '{1,2}'),
(1, 'group', '{1,2,3}');
