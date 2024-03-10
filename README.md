joe's little messaging app toy project

the server half isn't really portable as it requires a postgres server

if you wanna setup your own postgres server to work with it here's what the server is expecting

```
CREATE TABLE users (
id SERIAL PRIMARY KEY,
username varchar(64) UNIQUE NOT NULL,
password char(128) NOT NULL,
email varchar(254)
);
```

see `-h` for options to setup the postgres user, database name, path, etc.

the user must have access to the database and `users` table ofc, i use 

```
GRANT all ON users TO messaging_app_user;
```

i'll add actual messaging eventually, trying to think of a way to store everything that's slightly more efficient than one giant table with everyone's messages
