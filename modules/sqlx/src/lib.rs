// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko_core::*;
use sqlx::{pool::PoolOptions, Database};

pub struct SqlxModule<T: Database> {
  pub db_url: String,
  pub opts: PoolOptions<T>,
}

impl<T: Database> Default for SqlxModule<T> {
  fn default() -> Self {
    Self {
      db_url: std::env::var("DATABASEU_URL").expect("DATABASEU_URL is not present"),
      opts: PoolOptions::new(),
    }
  }
}

impl<T: Database> Module for SqlxModule<T> {
  fn init(&self, fw: &mut Framework) -> R {
    fw.runtime.push(|modules, state| {
      let sqlx = modules.take::<SqlxModule<T>>()?;
      Ok(Box::pin(async move {
        state
          .write()
          .await
          .put(sqlx.opts.connect(&sqlx.db_url).await?);
        Ok(None)
      }))
    });
    Ok(())
  }
}
