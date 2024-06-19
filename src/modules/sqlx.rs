// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use derivative::Derivative;
use sqlx::{postgres::PgPoolOptions, PgPool};

once_cell!(db, POOL: PgPool);

#[derive(Derivative)]
#[derivative(Default)]
pub struct Postgres {
  #[derivative(Default(value = "expect_env!(\"DATABASE_URL\")"))]
  pub db_url: String,
  pub options: PgPoolOptions,
}

impl Module for Postgres {
  async fn init(&mut self, fw: &mut Framework) -> R {
    runtime!(fw, |m| {
      POOL.set(m.options.connect(&m.db_url).await?)?;
      #[cfg(not(debug_assertions))]
      sqlx::migrate!("./sql").run(db()).await.unwrap();
      Ok(None)
    });
    Ok(())
  }
}
