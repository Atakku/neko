// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  modules::poise::Ctx,
  schema::*
};
use poise::AutocompleteChoice;
use sea_query::{Alias, Expr, Func, SelectStatement};

macro_rules! autocomplete {
  ( $fn_name:ident, $path:path) => {
    pub async fn $fn_name<'a>(ctx: Ctx<'_>, search: &'a str) -> Vec<AutocompleteChoice<String>> {
      use $path::*;
      let db = get_db_unwrap!(ctx.data());
      let mut qb = SelectStatement::new();
      qb.from(Table);
      qb.column(Name);
      qb.expr(Expr::col(Id)
      .cast_as(Alias::new("TEXT")));
      qb.and_where(
        Expr::expr(Func::lower(Expr::col(Name)))
          .like(format!("%{}%", search.to_lowercase()))
          .or(
            Expr::col(Id)
              .cast_as(Alias::new("TEXT"))
              .like(format!("%{search}%")),
          ),
      );
      qb.limit(25);
      fetch_all!(&db, &qb, (String, String))
        .unwrap_or(vec![])
        .into_iter()
        .map(|g| AutocompleteChoice {
          name: g.0,
          value: g.1,
        })
        .collect()
    }
  };
}

autocomplete!(discord_guilds, discord::Guilds);
autocomplete!(steam_apps, steam::Apps);
