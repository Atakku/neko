// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::time::SystemTime;

use crate::{core::*, schema::warnsys::*};
use sea_query::{Query, Expr};

const OFFSET: i64 = 60 * 60 * 24 * 7;

pub async fn active_user_warnings(id: i64) -> Res<Vec<(i64, i64, String)>> {
  let mut qb = Query::select();
  use Warnings::*;
  qb.from(Table);
  qb.columns([WarningId, DiscordId, Reason, IssuedAt]);
  qb.and_where(ex_col!(Warnings, DiscordId).eq(id));
  qb.and_where(ex_col!(Warnings, IssuedAt).gt(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64 - OFFSET));
  Ok(fetch_all!(&qb, (i64, i64, String))?)
}

pub async fn rm_warn(id: i64) -> Res<()> {
  use Warnings::*;
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(WarningId).eq(id));
  execute!(&qb)?;
  Ok(())
}

pub async fn add_user_warning(id: i64, reason: &String) -> Res<()> {
  use Warnings::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([DiscordId, Reason, IssuedAt]);
  qb.values([id.into(), reason.into(), (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64).into()])?;
  execute!(&qb)?;
  Ok(())
}
