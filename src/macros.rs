// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![allow(unused_macros)]

macro_rules! env {
  ($env:literal) => {
    std::env::var($env).expect(concat!($env, " is not present").into())
  };
  ($env:literal, $default:literal) => {
    std::env::var($env).unwrap_or($default.into())
  };
}

macro_rules! once_cell {
  (@define, $name:ident: $ty:ty) => {
    static $name: tokio::sync::OnceCell<$ty> = tokio::sync::OnceCell::const_new();
  };
  ($fun:ident, $name:ident: $ty:ty) => {
    once_cell!(@define, $name: $ty);

    pub fn $fun() -> &'static $ty {
      $name.get().expect(concat!(stringify!($fun), " has not yet been initialized"))
    }
  };
  ($fun:ident, $name:ident: $ty:ty, $block:block) => {
    once_cell!(@define, $name: $ty);

    pub async fn $fun() -> &'static $ty {
      $name.get_or_init(|| async $block).await
    }
  };
}

macro_rules! sql {
  (Prepare, $qb:expr) => {{
    (
      sea_query_binder::SqlxBinder::build_sqlx($qb, sea_query::PostgresQueryBuilder),
      crate::modules::sqlx::db(),
    )
  }};
  (FetchOne, $qb:expr, $ty:ty) => {{
    let ((q, v), p) = sql!(Prepare, $qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v).fetch_one(p).await
  }};
  (FetchAll, $qb:expr, $ty:ty) => {{
    let ((q, v), p) = sql!(Prepare, $qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v).fetch_all(p).await
  }};
  (FetchOpt, $qb:expr, $ty:ty) => {{
    let ((q, v), p) = sql!(Prepare, $qb);
    sqlx::query_as_with::<_, $ty, _>(&q, v)
      .fetch_optional(p)
      .await
  }};
  (Execute, $qb:expr) => {{
    let ((q, v), p) = sql!(Prepare, $qb);
    sqlx::query_with(&q, v).execute(p).await
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

macro_rules! autocomplete {
  ( $fn_name:ident, $path:path) => {
    pub async fn $fn_name<'a>(
      _: crate::modules::poise::Ctx<'_>,
      search: &'a str,
    ) -> Vec<poise::AutocompleteChoice<String>> {
      use sea_query::{Alias, Expr, Func, Order};
      use $path::*;
      let mut qb = sea_query::SelectStatement::new();
      qb.from(Table);
      qb.columns([Id, Name]);
      qb.and_where(
        Expr::expr(Func::lower(Expr::col(Name)))
          .like(format!("%{}%", search.to_lowercase()))
          .or(
            Expr::col(Id)
              .cast_as(Alias::new("TEXT"))
              .like(format!("%{search}%")),
          ),
      );
      qb.order_by(Name, Order::Asc);
      qb.limit(25);
      use unicode_truncate::UnicodeTruncateStr;
      sql!(FetchAll, &qb, (i64, String))
        .unwrap_or(vec![])
        .into_iter()
        .map(|g| poise::AutocompleteChoice {
          value: g.0.to_string(),
          name: g.1.unicode_truncate(100).0.into(),
        })
        .collect()
    }
  };
}

macro_rules! schema {
  ($(#[table($table:literal)] $vis:vis enum $ident:ident {$($field:ident),*$(,)?})*) => {
    $(
      #[derive(sea_query::Iden)]
      #[iden(rename = $table)]
      #[allow(dead_code)]
      $vis enum $ident {
        Table, $($field),*
      }
    )*
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

macro_rules! rt {
  ($fw:ident, |$m:ident| $block:block) => {
    $fw.runtime.push(|modules| {
      let $m = modules.take::<Self>()?;
      Ok(Box::pin(async move $block))
    });
  };
}

macro_rules! col {
  ($path:path, $ident:ident) => {{
    use $path::*;
    (Table, $ident)
  }};
}

macro_rules! ex_col {
  ($path:path, $ident:ident) => {
    sea_query::Expr::col(col!($path, $ident))
  };
}
