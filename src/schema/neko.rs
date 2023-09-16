// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "neko_users")]
pub enum Users {
  Table,
  Id,
  Slug,
}

#[derive(Iden)]
#[iden(rename = "neko_connections_discord")]
pub enum Discord {
  Table,
  Id,
  DiscordId,
}

#[derive(Iden)]
#[iden(rename = "neko_connections_steam")]
pub enum Steam {
  Table,
  Id,
  SteamId,
}
