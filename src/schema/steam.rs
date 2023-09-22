// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "steam_users")]
pub enum Users {
  Table,
  Id,
  Name,
  Avatar,
  LastOnline,
}

#[derive(Iden)]
#[iden(rename = "steam_apps")]
pub enum Apps {
  Table,
  Id,
  Name,
}

#[derive(Iden)]
#[iden(rename = "steam_playdata")]
pub enum Playdata {
  Table,
  Id,
  UserId,
  AppId,
  Playtime,
}

#[derive(Iden)]
#[iden(rename = "steam_playdata_history")]
pub enum PlaydataHistory {
  Table,
  PlaydataId,
  UtcDay,
  Playtime,
}


#[derive(Iden)]
#[iden(rename = "steam_discord_roles")]
pub enum DiscordRoles {
  Table,
  GuildId,
  RoleId,
  AppId,
}

