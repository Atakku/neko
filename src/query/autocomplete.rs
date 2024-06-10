// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  modules::poise::Ctx,
  schema::*
};
use poise::AutocompleteChoice;
use sea_query::{Alias, Expr, Func, SelectStatement, Order};

macro_rules! autocomplete {
  ( $fn_name:ident, $path:path) => {
    pub async fn $fn_name<'a>(_: Ctx<'_>, search: &'a str) -> Vec<AutocompleteChoice<String>> {
      use $path::*;
      let mut qb = SelectStatement::new();
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
      fetch_all!(&qb, (i64, String))
        .unwrap_or(vec![])
        .into_iter()
        .map(|g| AutocompleteChoice {
          value: g.0.to_string(),
          name: g.1.unicode_truncate(100).0.into(),
        })
        .collect()
    }
  };
}

autocomplete!(discord_guilds, discord::Guilds);
autocomplete!(steam_apps, steam::Apps);


pub async fn findr_cities<'a>(_: Ctx<'_>, search: &'a str) -> Vec<AutocompleteChoice<String>> {
  use findr::Cities::*;
  let mut qb = SelectStatement::new();
  qb.from(Table);
  qb.columns([Id, City, Region, Country]);
  qb.and_where(
    Expr::expr(Func::lower(Expr::col(City)))
      .like(format!("%{}%", search.to_lowercase()))
      .or(
        Expr::col(Id)
          .cast_as(Alias::new("TEXT"))
          .like(format!("%{search}%")),
      ),
  );
  qb.order_by(City, Order::Asc);
  qb.limit(25);
  use unicode_truncate::UnicodeTruncateStr;
  fetch_all!(&qb, (i64, String, Option<String>, String))
    .unwrap_or(vec![])
    .into_iter()
    .map(|g| AutocompleteChoice {
      value: g.0.to_string(),
      name: (format!("{} | {} | {}", g.1, g.2.unwrap_or(g.3.clone()), g.3)).unicode_truncate(100).0.into(),
    })
    .collect()
}