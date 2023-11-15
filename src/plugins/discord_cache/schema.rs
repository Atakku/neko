// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  pub enum DiscordCacheGuilds {
    GuildId.big_integer().primary_key(),
    Name.string_len(100).not_null(),
    Icon.string_len(34),
  }

  pub enum DiscordCacheUsers {
    UserId.big_integer().primary_key(),
    Name.string_len(32).not_null(),
    Nick.string_len(32),
    Avatar.string_len(34),
  }

  pub enum DiscordCacheMembers {
    GuildId.big_integer().not_null(),
    UserId.big_integer().not_null(),
    Nick.string_len(32),
    Avatar.string_len(34);

    Self.foreign_key(fk!(DiscordCacheGuilds, GuildId, Cascade, Cascade))
      .foreign_key(fk!(DiscordCacheUsers, UserId, Cascade, Cascade))
      .primary_key(pk!(GuildId, UserId))
  }
}
