// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("discord_roles_steam")]
  pub enum SteamDiscordRoles {
    GuildId.big_integer(), 
    RoleId.big_integer(), 
    AppId.big_integer(),
  }
}
