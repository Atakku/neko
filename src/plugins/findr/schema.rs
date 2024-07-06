// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use sea_query::Iden;

#[derive(Iden)]
#[iden(rename = "finder_cities")]
pub enum Cities {
  Table,
  Id,
  Lat,
  Lng,
  City,
  Region,
  Country
}

#[derive(Iden)]
#[iden(rename = "finder_users")]
pub enum Users {
  Table,
  UserId,
  CityId
}
