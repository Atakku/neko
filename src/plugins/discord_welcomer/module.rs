// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::modules::{poise::Poise, sqlx::Postgres};

module! {
  DiscordWelcomer;

  fn init(fw) {
    fw.req::<Postgres>()?;
    fw.req::<Poise>()?;
  }
}
