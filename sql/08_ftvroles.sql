CREATE TABLE ftvr_categories (
  id ftvr_roles PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  title TEXT NOT NULL
);

CREATE TABLE ftvr_roles (
  category_id INTEGER NOT NULL REFERENCES ftvr_categories(id) ON DELETE CASCADE ON UPDATE CASCADE,
  role_id BIGINT NOT NULL,
  emote TEXT NOT NULL,
  custom_emote BOOLEAN NOT NULL,
  title TEXT NOT NULL
);
