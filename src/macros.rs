// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

macro_rules! get_state {
  ( $ctx:expr ) => {
    $ctx.read().await
  };
}

macro_rules! state_clone {
  ($ctx:expr, $p:path) => {
    get_state!($ctx).borrow::<$p>()?.clone()
  };
}

macro_rules! state_clone_unwrap {
  ($ctx:expr, $p:path) => {
    get_state!($ctx).borrow::<$p>().unwrap().clone()
  };
}

macro_rules! get_db {
  ( $ctx:expr ) => {
    state_clone!($ctx, sqlx::PgPool)
  };
}

macro_rules! get_db_unwrap {
  ( $ctx:expr ) => {
    state_clone_unwrap!($ctx, sqlx::PgPool)
  };
}

macro_rules! get_reqwest {
  ( $ctx:expr ) => {
    state_clone!($ctx, reqwest::Client)
  };
}

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
  ( $db:expr, $qb:expr, $t:tt ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $t, _>(&q, v).fetch_one($db).await
  }};
}
macro_rules! fetch_all {
  ( $db:expr, $qb:expr, $t:tt ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $t, _>(&q, v).fetch_all($db).await
  }};
}
macro_rules! execute {
  ( $db:expr, $qb:expr ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_with(&q, v).execute($db).await
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
