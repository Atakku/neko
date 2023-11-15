// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  pub enum DiscordRolesSteam {
    GuildId.big_integer(),
    RoleId.big_integer().unique_key(),
    AppId.big_integer();
    Self.primary_key(pk!(GuildId, AppId))
  }
}
