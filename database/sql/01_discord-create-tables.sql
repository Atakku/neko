CREATE TABLE discord_guilds (
  id BIGINT PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  icon VARCHAR(34)
);

CREATE TABLE discord_users (
  id BIGINT PRIMARY KEY,
  username VARCHAR(32) NOT NULL,
  nickname VARCHAR(32),
  avatar VARCHAR(34)
);

CREATE TABLE discord_members (
  guild_id BIGINT NOT NULL REFERENCES discord_guilds(id) ON DELETE CASCADE ON UPDATE CASCADE,
  user_id BIGINT NOT NULL REFERENCES discord_users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  nickname VARCHAR(32),
  avatar VARCHAR(34),
  PRIMARY KEY (guild_id, user_id)
);