// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use sqlx::postgres::PgPoolOptions;

pub struct Postgres {
  db_url: String,
  options: PgPoolOptions,
}

impl Default for Postgres {
  fn default() -> Self {
    Self {
      db_url: expect_env!("DATABASE_URL"),
      options: PgPoolOptions::new(),
    }
  }
}

once_cell!(db, POOL: sqlx::PgPool);

impl Module for Postgres {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|m| {
      let this = m.take::<Self>()?;
      rt_async!({
        POOL.set(this.options.connect(&this.db_url).await?)?;
        sqlx::migrate!("./sql").run(db()).await.unwrap();
        Ok(None)
      })
    });
    Ok(())
  }
}
