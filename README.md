### environment variables
app.yaml
server:
 - port:
 ...

### some commands
- mv * chat_server/   move some files to chat_server directory
- mv _typos.toml ../  move _typos.toml to last directory
- cat index.html  search some info about index.html

### make a project change into a workspace
- write some info to the Cargo.toml something like this:
- [workspace]
  members = ["chat_server", "notify_server"]
  resolver = "2"

  [workspace.dependencies]
  anyhow = "1.0.93"

### postgresql
- if you have installed the postgresql in your system, you can use some inner command, like this:
- createdb database   create a database
  dropdb database     drop a database
  psql                serve the postgresql

### install sqlx cli only for postgresql
- cargo install sqlx-cli --no-default-features --features rustls --features postgres
- then run ```sqlx```   search some commands about sqlx-cli

### sqlx cli commands
- ```slqx migrate add initial.sql```
  (note: name is optional)    (add a sql, create a migrations directory)

- ```sqlx migrate run ``` (run migrations directory)

- `sqlx migrate run` this command only one to use, and if you want to update a sql such as
   initial.sql,  you can use `sqlx migrate run` this command to create a new sql and alter something
   about initial.sql rather than to direct update initial.sql

### argon2 password hash  lattery generate hash
- Slow hash generation to hash password brute-force attacks
- argon2 hash's length is 97
