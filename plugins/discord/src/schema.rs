// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "discord_guilds")]
pub enum Guilds {
  Table,
  Id,
  Name,
  Icon,
}

#[derive(Iden)]
#[iden(rename = "discord_users")]
pub enum Users {
  Table,
  Id,
  Username,
  Nickname,
  Avatar,
}

#[derive(Iden)]
#[iden(rename = "discord_members")]
pub enum Members {
  Table,
  GuildId,
  UserId,
  Nickname,
  Avatar,
}
