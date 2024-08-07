// Copyright 2024 Atakku <https://atakku.dev>
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
  Name,
  Nick,
  Avatar,
}

#[derive(Iden)]
#[iden(rename = "discord_members")]
pub enum Members {
  Table,
  GuildId,
  UserId,
  Nick,
  Avatar,
}
