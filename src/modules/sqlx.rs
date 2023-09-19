// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use sqlx::postgres::PgPoolOptions;

/// Sqlx wrapper module for connecting to a postgres database
pub struct Postgres {
  db_url: String,
  options: PgPoolOptions,
}

impl Default for Postgres {
  fn default() -> Self {
    Self {
      db_url: std::env::var("DATABASE_URL").expect("DATABASE_URL is not present"),
      options: PgPoolOptions::new(),
    }
  }
}

impl Module for Postgres {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|mds, rt| {
      let postgres = mds.take::<Self>()?;
      Ok(Box::pin(async move {
        let pool = postgres.options.connect(&postgres.db_url).await?;
        sqlx::migrate!("./sql").run(&pool).await.unwrap();
        rt.write().await.put(pool);
        Ok(None)
      }))
    });
    Ok(())
  }
}
