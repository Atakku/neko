// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use chrono::{DateTime, Utc};
use serde::Deserialize;

api!(DeepRockGalacticApi, "https://drgapi.com/v1/", {
  fn get_deepdives("deepdives") -> DeepDives;
  fn get_salutes("salutes") -> Salutes;
  fn get_trivia("trivia") -> Trivia;
});

#[derive(Deserialize)]
pub struct Salutes {
  pub salutes: Vec<String>,
}

#[derive(Deserialize)]
pub struct Trivia {
  pub trivia: Vec<String>,
}

#[derive(Deserialize)]
pub struct DeepDives {
  #[serde(rename = "startTime")]
  pub start_time: DateTime<Utc>,
  #[serde(rename = "endTime")]
  pub end_time: DateTime<Utc>,
  pub variants: Vec<Variant>,
}

#[derive(Deserialize)]
pub struct Variant {
  #[serde(rename = "type")]
  pub dive_type: DeepdiveType,
  pub name: String,
  pub biome: String,
  pub seed: i64,
  pub stages: Vec<Stage>,
}

#[derive(Deserialize)]
pub struct Stage {
  pub id: i32,
  pub primary: String,
  pub secondary: String,
  #[serde(rename = "anomaly")]
  pub mutator: Option<String>,
  pub warning: Option<String>,
}

#[derive(Deserialize)]
pub enum DeepdiveType {
  #[serde(rename = "Deep Dive")]
  DeepDive,
  #[serde(rename = "Elite Deep Dive")]
  EliteDeepDive,
}
