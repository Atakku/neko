// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

schema! {
  #[table("steam_users")]
  pub enum Users {
    Id, Name, Avatar, LastOnline,
  }

  #[table("steam_apps")]
  pub enum Apps {
    Id, Name,
  }

  #[table("steam_playdata")]
  pub enum Playdata {
    Id, UserId, AppId, Playtime,
  }

  #[table("steam_playdata_history")]
  pub enum PlaydataHistory {
    PlaydataId, UtcDay, Playtime,
  }

  #[table("steam_discord_roles")]
  pub enum DiscordRoles {
    GuildId, RoleId, AppId,
  }
}
