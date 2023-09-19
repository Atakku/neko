// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![allow(unused_macros)]

macro_rules! expect_env {
  ($env:literal) => {
    std::env::var($env).expect(concat!($env, " is not present").into())
  };
}
macro_rules! default_env {
  ($env:literal, $default:literal) => {
    std::env::var($env).unwrap_or($default.into())
  };
}

macro_rules! init_once {
  ($fn:ident, $name:ident: $ty:ty, $block:block) => {
    static $name: tokio::sync::OnceCell<$ty> = tokio::sync::OnceCell::const_new();

    pub async fn $fn() -> &'static $ty {
      $name
      .get_or_init(|| async $block)
      .await
    }

    macro_rules! $fn {
      () => {
        crate::macros::$fn().await
      }
    }
  };
}

init_once!(reqwest, CLIENT: reqwest::Client, {
  let ua = default_env!("USER_AGENT", "neko.rs");
  reqwest::Client::builder().user_agent(ua).build().unwrap()
});

init_once!(db, DB: sqlx::PgPool, {
  let pool = sqlx::postgres::PgPoolOptions::new().connect(&expect_env!("DATABASE_URL")).await.unwrap();
  sqlx::migrate!("./sql").run(&pool).await.unwrap();
  pool
});

init_once!(loc, LOCALE: crate::modules::fluent::FluentBundles, {
  crate::modules::fluent::init().unwrap()
});

macro_rules! rt_async {
    ($block:block) => {
      Ok(Box::pin(async move $block))
    };
}
macro_rules! build_sqlx {
  ($qb:expr) => {
    sea_query_binder::SqlxBinder::build_sqlx($qb, sea_query::PostgresQueryBuilder)
  };
}
macro_rules! fetch_one {
  ( $qb:expr, $t:tt ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $t, _>(&q, v)
      .fetch_one(db!())
      .await
  }};
}
macro_rules! fetch_all {
  ( $qb:expr, $t:tt ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $t, _>(&q, v)
      .fetch_all(db!())
      .await
  }};
}
macro_rules! execute {
  ( $qb:expr ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_with(&q, v).execute(db!()).await
  }};
}

macro_rules! api {
  ($name:ident, $base:literal, {
    $(
      fn $fun:ident($endpoint:literal) -> $ty:ty $({
        $($pn:ident:$pt:ty),*$(,)?
      })?;
    )*
  }) => {
    pub trait $name {
      $(async fn $fun(&self, $($($pn: $pt),*)?) -> crate::core::Res<$ty>;)*
    }

    impl $name for reqwest::Client {
      $(async fn $fun(&self, $($($pn: $pt),*)?) -> crate::core::Res<$ty> {
        Ok(self.get(format!(concat!($base, $endpoint, $("?",$(stringify!($pn), "={", stringify!($pn), "}&"),*)?), $($($pn=$pn),*)?)).send().await?.json::<$ty>().await?)
      })*
    }
  };
}
