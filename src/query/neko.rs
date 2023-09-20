// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, schema::neko::*};
use sea_query::Query;

pub async fn all_steam_connections() -> Res<Vec<(i64,)>> {
  let mut qb = Query::select();
  qb.from(Steam::Table);
  qb.column(Steam::SteamId);
  Ok(fetch_all!(&qb, (i64,))?)
}
