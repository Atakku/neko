// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "poststreak")]
pub enum Posts {
  Table,
  UserId,
  Timestamp,
  Streak
}

