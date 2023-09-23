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

macro_rules! once_cell {
  ($fun:ident, $name:ident: $ty:ty) => {
    static $name: tokio::sync::OnceCell<$ty> = tokio::sync::OnceCell::const_new();

    pub fn $fun() -> &'static $ty {
      $name
        .get()
        .expect(concat!(stringify!($fun), " has not yet been initialized"))
    }
  };
}

macro_rules! build_sqlx {
  ($qb:expr) => {
    sea_query_binder::SqlxBinder::build_sqlx($qb, sea_query::PostgresQueryBuilder)
  };
}

macro_rules! fetch_optional {
  ( $qb:expr, $ty:ty ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v)
      .fetch_optional(crate::modules::sqlx::db())
      .await
  }};
}
macro_rules! fetch_one {
  ( $qb:expr, $ty:ty ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v)
      .fetch_one(crate::modules::sqlx::db())
      .await
  }};
}
macro_rules! fetch_all {
  ( $qb:expr, $ty:ty ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v)
      .fetch_all(crate::modules::sqlx::db())
      .await
  }};
}
macro_rules! execute {
  ( $qb:expr ) => {{
    let (q, v) = build_sqlx!($qb);
    sqlx::query_with(&q, v)
      .execute(crate::modules::sqlx::db())
      .await
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
        let req = format!(concat!($base, $endpoint, $("?",$(stringify!($pn), "={", stringify!($pn), "}&"),*)?), $($($pn=$pn),*)?);
        log::trace!("Sending req to {req}");
        let res = self.get(req).send().await?;
        log::trace!("Received status: {}", res.status());
        Ok(res.json::<$ty>().await?)
      })*
    }
  };
}

macro_rules! not_match {
  ( $v:expr, $pat:pat, $block:block ) => {
    match $v {
      $pat => {}
      _ => $block,
    }
  };
}

macro_rules! cmd_group {
  ($cmd:ident, $($sub:literal),*) => {
    #[poise::command(prefix_command, slash_command, subcommand_required, subcommands($($sub),*))]
    pub async fn $cmd(_: crate::modules::poise::Ctx<'_>) -> crate::core::R {Ok(())}
  };
}

macro_rules! runtime {
  ($fw:ident, |$m:ident| $block:block) => {
    $fw.runtime.push(|modules| {
      let $m = modules.take::<Self>()?;
      Ok(Box::pin(async move $block))
    });
  };
}

macro_rules! module {
  ($name:ident {$($pn:ident: $pt:ty = $pd:expr),*$(,)?} fn init($fw:ident) $block:block) => {
    pub struct $name {
      $(pub $pn: $pt),*
    }

    impl Default for $name {
      fn default() -> Self { 
        Self {
          $($pn: $pd),*
        }
      }
    }

    impl crate::core::Module for $name {
      fn init(&mut self, $fw: &mut crate::core::Framework) -> crate::core::R {
        $block
        Ok(())
      }
    }
  };
}
 

macro_rules! col {
  ($path:path, $ident:ident) => {
    {
      use $path::*;
      (Table, $ident)
    }
  };
}

macro_rules! ex_col {
  ($path:path, $ident:ident) => {
    sea_query::Expr::col(col!($path, $ident))
  };
}
