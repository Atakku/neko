// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("steam_users")]
  pub enum SteamUsers {
    Id, Name, Avatar, LastOnline,
  }

  #[table("steam_apps")]
  pub enum SteamApps {
    Id, Name,
  }

  #[table("steam_playdata")]
  pub enum SteamPlaydata {
    Id, UserId, AppId, Playtime,
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
