// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("discord_welcomer")]
  pub enum DiscordWelcomer {
    GuildId, 
    ChannelId
  }
}
