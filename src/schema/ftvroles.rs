// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "ftvr_categories")]
pub enum Category {
  Table,
  Id,
  Title,
}

#[derive(Iden)]
#[iden(rename = "ftvr_roles")]
pub enum Roles {
  Table,
  CategoryId,
  RoleId,
  Emote,
  CustomEmote,
  Title,
}
