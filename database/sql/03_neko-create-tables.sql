CREATE TABLE neko_users (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  slug VARCHAR(32) UNIQUE
);

CREATE TABLE neko_connections_discord (
  id INTEGER PRIMARY KEY,
  discord_id BIGINT UNIQUE NOT NULL REFERENCES discord_users(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE neko_connections_steam (
  id INTEGER PRIMARY KEY,
  steam_id BIGINT UNIQUE NOT NULL REFERENCES steam_users(id) ON DELETE CASCADE ON UPDATE CASCADE
);