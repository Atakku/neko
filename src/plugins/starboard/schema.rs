// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "starboard_posts")]
pub enum Posts {
  Table,
  SourceId,
  PostId,
}

