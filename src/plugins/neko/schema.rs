// Copyright 2024 Atakku <https://atakku.dev>
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
#[iden(rename = "neko_users_discord")]
pub enum UsersDiscord {
  Table,
  NekoId,
  DiscordId,
}

#[derive(Iden)]
#[iden(rename = "neko_users_steam")]
pub enum UsersSteam {
  Table,
  NekoId,
  SteamId,
}

#[derive(Iden)]
#[iden(rename = "neko_users_github")]
pub enum UsersGithub {
  Table,
  NekoId,
  GithubId,
}

#[derive(Iden)]
#[iden(rename = "neko_users_anilist")]
pub enum UsersAnilist {
  Table,
  NekoId,
  AnilistId,
}

#[derive(Iden)]
#[iden(rename = "neko_users_telegram")]
pub enum UsersTelegram {
  Table,
  NekoId,
  TelegramId,
}

#[derive(Iden)]
#[iden(rename = "neko_whitelist_discord")]
pub enum WhitelistDiscord {
  Table,
  GuildId,
}
