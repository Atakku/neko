// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "starboard_posts")]
pub enum Posts {
  Table,
  SouceId,
  PostId,
}

