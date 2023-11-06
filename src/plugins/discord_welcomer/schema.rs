// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("discord_welcomer")]
  pub enum SteamDiscordRoles {
    GuildId.big_integer().primary_key(), 
    ChannelId.big_integer()
  }
}
