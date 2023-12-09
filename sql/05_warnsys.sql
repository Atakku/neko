CREATE TABLE warnsys_warnings (
  warning_id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  discord_id BIGINT NOT NULL,
  reason TEXT NOT NULL,
  issued_at BIGINT NOT NULL
);