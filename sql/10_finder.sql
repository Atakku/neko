CREATE TABLE finder_cities (
  id BIGINT PRIMARY KEY,
  lat FLOAT NOT NULL,
  lng FLOAT NOT NULL,
  city TEXT NOT NULL,
  region TEXT,
  country TEXT NOT NULL
);

CREATE TABLE finder_users (
  user_id BIGINT PRIMARY KEY,
  city_id BIGINT NOT NULL REFERENCES finder_cities(id) ON DELETE CASCADE ON UPDATE CASCADE
);
