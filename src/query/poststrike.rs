// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::schema::poststrike::*;
use crate::core::*;
use sea_query::{OnConflict, Query};

pub async fn update_timestamp(user_id: i64, timestamp: i64, strike: i64) -> Res<()> {
  use Posts::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([UserId, Timestamp, Strike]);
  qb.on_conflict(
    OnConflict::column(UserId)
      .update_columns([Timestamp, Strike])
      .to_owned(),
  );
  qb.values([user_id.into(), timestamp.into(), strike.into()])?;
  execute!(&qb)?;
  Ok(())
}

pub async fn get_strike(user_id: i64) -> Res<Option<(i64, i64)>> {
  use Posts::*; 
  let mut qb = Query::select();
  qb.from(Table);
  qb.columns([Timestamp, Strike]);
  qb.and_where(ex_col!(Posts, UserId).eq(user_id));
  Ok(fetch_optional!(&qb, (i64, i64))?) 
}
