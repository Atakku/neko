// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("discord_guilds")]
  pub enum DiscordGuilds {
    Id, Name, Icon,
  }

  #[table("discord_users")]
  pub enum DiscordUsers {
    Id, Name, Nick, Avatar,
  }

  #[table("discord_members")]
  pub enum DiscordMembers {
    GuildId, UserId, Nick, Avatar,
  }
}
