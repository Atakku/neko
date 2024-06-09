// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::{OnConflict, Query};

use crate::{core::Res, schema::finder::Users};

pub async fn update_city(user_id: i64, city_id: i64) -> Res<()> {
  use Users::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([UserId, CityId]);
  qb.on_conflict(OnConflict::column(UserId).update_column(CityId).to_owned());
  qb.values([user_id.into(), city_id.into()])?;
  execute!(&qb)?;
  Ok(())
}
