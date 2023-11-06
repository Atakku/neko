// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("steam_users")]
  pub enum SteamUsers {
    SteamId.big_integer().primary_key(),
    Username.text(),
    Avatar.text(),
    LastOnline.big_integer(),
  }

  #[table("steam_apps")]
  pub enum SteamApps {
    AppId.big_integer().primary_key(),
    AppName.text(),
  }

  #[table("steam_playdata")]
  pub enum SteamPlaydata {
    // TODO: identity
    PlaydataId.big_integer().primary_key(),
    UserId.big_integer().not_null(),
    AppId.big_integer().not_null(),
    Playtime.integer().not_null(),
  }

  #[table("steam_playhist")]
  pub enum SteamPlayhist {
    PlaydataId, UtcDay, Playtime,
  }

  // TODO: Move to discord? or separate
  #[table("steam_discord_roles")]
  pub enum SteamDiscordRoles {
    GuildId, RoleId, AppId,
  }
}
