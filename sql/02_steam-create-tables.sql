CREATE TABLE steam_users (
  id BIGINT PRIMARY KEY,
  name TEXT,
  avatar TEXT,
  last_online BIGINT
);

CREATE TABLE steam_apps (
  id BIGINT PRIMARY KEY, 
  name TEXT
);

CREATE TABLE steam_playdata (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  user_id BIGINT NOT NULL REFERENCES steam_users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  app_id BIGINT NOT NULL REFERENCES steam_apps(id) ON DELETE CASCADE ON UPDATE CASCADE,
  playtime INTEGER NOT NULL,
  UNIQUE (user_id, app_id)
);

CREATE TABLE steam_playdata_history (
  playdata_id BIGINT NOT NULL REFERENCES steam_playdata(id) ON DELETE CASCADE ON UPDATE CASCADE,
  utc_day INTEGER NOT NULL,
  playtime INTEGER NOT NULL,
  PRIMARY KEY (playdata_id, utc_day)
);

CREATE TABLE steam_discord_roles (
  guild_id BIGINT,
  role_id BIGINT UNIQUE,
  app_id BIGINT,
  PRIMARY KEY (guild_id, app_id)
);
