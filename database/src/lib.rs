// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko_core::*;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{
  postgres::{PgPoolOptions, PgQueryResult, PgRow},
  FromRow, PgPool,
};
use tokio::sync::OnceCell;

pub mod discord;
pub mod steam;

static DB: OnceCell<PgPool> = OnceCell::const_new();

pub async fn get_db<'a>() -> &'a PgPool {
  DB.get_or_init(|| async {
    let pool = PgPoolOptions::new()
      .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL is not present"))
      .await
      .unwrap();
    sqlx::migrate!("./sql").run(&pool).await.unwrap();
    pool
  })
  .await
}

pub async fn fetch_one<T>(qb: impl SqlxBinder) -> Res<T>
where
  T: std::marker::Send,
  T: std::marker::Unpin,
  T: for<'r> FromRow<'r, PgRow>,
{
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  Ok(
    sqlx::query_as_with::<_, T, _>(&q, v)
      .fetch_one(get_db().await)
      .await?,
  )
}

pub async fn fetch_all<Q, T>(qb: Q) -> Res<Vec<T>>
where
  Q: SqlxBinder,
  T: std::marker::Send,
  T: std::marker::Unpin,
  T: for<'r> FromRow<'r, PgRow>,
{
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  Ok(sqlx::query_as_with::<_, T, _>(&q, v)
    .fetch_all(get_db().await)
    .await?)
}

pub async fn execute<Q: SqlxBinder>(qb: Q) -> Res<PgQueryResult> {
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  Ok(sqlx::query_with(&q, v).execute(get_db().await).await?)
}
