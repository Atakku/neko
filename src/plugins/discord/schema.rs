// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("discord_guilds")]
  pub enum Guilds {
    Id, Name, Icon,
  }

  #[table("discord_users")]
  pub enum Users {
    Id, Name, Nick, Avatar,
  }

  #[table("discord_members")]
  pub enum Members {
    GuildId, UserId, Nick, Avatar,
  }
}
