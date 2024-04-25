// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, schema::starboard::*};
use sea_query::{OnConflict, Query};

pub async fn get_post_id(source_id: i64) -> Res<Option<(i64,)>> {
  use Posts::*; 
  let mut qb = Query::select();
  qb.from(Table);
  qb.columns([PostId]);
  qb.and_where(ex_col!(Posts, SouceId).eq(source_id));
  Ok(fetch_optional!(&qb, (i64,))?) 
}

pub async fn upsert_post(source_id: i64, post_id: i64) -> Res<()> {
  use Posts::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([SouceId, PostId]);
  qb.on_conflict(
    OnConflict::column(SouceId)
      .update_columns([PostId])
      .to_owned(),
  );
  qb.values([source_id.into(), post_id.into()])?;
  execute!(&qb)?;
  Ok(())
}
