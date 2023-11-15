// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::modules::{poise::Poise, sqlx::Postgres};

module! {
  DiscordRoles;

  fn init(fw) {
    let pg = fw.req::<Postgres>()?;
    pg.create_tables(&mut super::schema::create_tables());
    fw.req::<Poise>()?;
  }
}
