// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::schema::*;
use crate::core::*;
use sea_query::{OnConflict, Query};

pub async fn update_timestamp(user_id: i64, timestamp: i64, streak: i64) -> Res<()> {
  use Posts::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([UserId, Timestamp, Streak]);
  qb.on_conflict(
    OnConflict::column(UserId)
      .update_columns([Timestamp, Streak])
      .to_owned(),
  );
  qb.values([user_id.into(), timestamp.into(), streak.into()])?;
  execute!(&qb)?;
  Ok(())
}

pub async fn get_streak(user_id: i64) -> Res<Option<(i64, i64)>> {
  use Posts::*; 
  let mut qb = Query::select();
  qb.from(Table);
  qb.columns([Timestamp, Streak]);
  qb.and_where(ex_col!(Posts, UserId).eq(user_id));
  Ok(fetch_optional!(&qb, (i64, i64))?) 
}
