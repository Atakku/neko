// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{
  postgres::{PgPoolOptions, PgQueryResult, PgRow},
  query_as_with, query_with, FromRow, PgPool,
};

use crate::core::*;

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
  fn init(&self, fw: &mut Framework) -> R {
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

pub trait PgOut = Send + Unpin + for<'r> FromRow<'r, PgRow>;

pub trait PgHelper {
  async fn fetch_one<T: PgOut>(&self, qb: impl SqlxBinder) -> Res<T>;
  async fn fetch_all<T: PgOut>(&self, qb: impl SqlxBinder) -> Res<Vec<T>>;
  async fn execute(&self, qb: impl SqlxBinder) -> Res<PgQueryResult>;
}

impl PgHelper for PgPool {
  async fn fetch_one<T: PgOut>(&self, q: impl SqlxBinder) -> Res<T> {
    let (q, v) = q.build_sqlx(PostgresQueryBuilder);
    Ok(query_as_with::<_, T, _>(&q, v).fetch_one(self).await?)
  }

  async fn fetch_all<T: PgOut>(&self, q: impl SqlxBinder) -> Res<Vec<T>> {
    let (q, v) = q.build_sqlx(PostgresQueryBuilder);
    Ok(query_as_with::<_, T, _>(&q, v).fetch_all(self).await?)
  }

  async fn execute(&self, q: impl SqlxBinder) -> Res<PgQueryResult> {
    let (q, v) = q.build_sqlx(PostgresQueryBuilder);
    Ok(query_with(&q, v).execute(self).await?)
  }
}
