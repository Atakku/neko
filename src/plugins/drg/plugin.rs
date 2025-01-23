// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  plugins::drg::interface::{DeepRockGalacticApi, Variant},
  modules::{
    poise::{Ctx, Poise},
    reqwest::{req, Reqwest},
  },
};
use std::fmt;

pub mod interface;

pub struct DeepRockGalactic;

impl Module for DeepRockGalactic {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Reqwest>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(drg());
    Ok(())
  }
}
#[poise::command(prefix_command, slash_command)]
pub async fn drg(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Fetching Deep Dives...").await?;
  let res = req().get_deepdives().await?;
  m.edit(ctx, |m| {
    m.embed(|e| {
      for variant in res.variants {
        let va = variant.to_string();
        e.field(
          format!("{} {}", variant.name, biome_icon(variant.biome)),
          va,
          true,
        );
      }
      e
    })
  })
  .await?;
  Ok(())
}

fn biome_icon(biome: String) -> String {
  match biome.as_str() {
    "Crystalline Caverns" => "<:biome_crystalline_caverns:1152814710819913788>",
    "Salt Pits" => "<:biome_salt_pits:1152814715714682950>",
    "Fungus Bogs" => "<:biome_fungus_bogs:1152814720521343076>",
    "Radioactive Exclusion Zone" => "<:biome_radioactive_exclusion_zone:1152814726624055336>",
    "Dense Biozone" => "<:biome_dense_biozone:1152814733016170588>",
    "Glacial Strata" => "<:biome_glacial_strata:1152814738586222592>",
    "Hollow Bough" => "<:biome_hollow_bough:1152814742398828654>",
    "Azure Weald" => "<:biome_azure_weald:1152814746005930074>",
    "Magma Core" => "<:biome_magma_core_icon:1152814749915029545>",
    "Sandblasted Corridors" => "<:biome_sandblasted_corridors:1152814752293191802>",
    _ => "",
  }
  .into()
}

fn mutator_icon(mutator: String) -> String {
  match mutator.as_str() {
    "Critical Weakness" => "<:mutator_critical_weakness:1152800323967135856>",
    "Double XP" => "<:mutator_double_xp:1152800395341598810>",
    "Gold Rush" => "<:mutator_gold_rush:1152800396918661130>",
    "Golden Bugs" => "<:mutator_golden_bugs:1152800399150035095>",
    "Low Gravity" => "<:mutator_low_gravity:1152800400651591760>",
    "Mineral Mania" => "<:mutator_mineral_mania:1152800403407257601>",
    "Rich Atmosphere" => "<:mutator_rich_atmosphere:1152800405005275186>",
    "Volatile Guts" => "<:mutator_volatile_guts:1152800406691381340> ",
    _ => "",
  }
  .into()
}

fn warning_icon(warning: String) -> String {
  match warning.as_str() {
    "Cave Leech Cluster" => "<:warn_cave_leech_cluster:1330838379499360266>",
    "Elite Threat" => "<:warn_elite_threat:1330839232864059392>",
    "Exploder Infestation" => "<:warn_exploder_infestation:1330839222114058260>",
    "Haunted Cave" => "<:warn_haunted_cave:1330839171341750272>",
    "Lethal Enemies" => "<:warn_lethal_enemies:1330839161665617972>",
    "Low Oxygen" => "<:warn_low_oxygen:1330839123040272384>",
    "Mactera Plague" => "<:warn_mactera_plague:1330839114240491531>",
    "Parasites" => "<:warn_parasites:1330839103700336660>",
    "Regenerative Bugs" => "<:warn_regenerative_bugs:1330839021223678038>",
    "Rival Presence" => "<:warn_rival_presence:1330838921784856640>",
    "Shield Disruption" => "<:warn_shield_disruption:1330838830173130783>",
    "Swarmageddon" => "<:warn_swarmageddon:1330838695947014174>",
    "Bulk Infestation" => "<:warn_bulk_infestation:1330839188475744322>",
    "Lithophage Outbreak" => "<:warn_lithophage_outbreak:1330839133500739595>",
    _ => "<:warning_placeholder:1152827809983631390>",
  }
  .into()
}
impl fmt::Display for Variant {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let stages = &self
      .stages
      .iter()
      .map(|s| {
        let warning = s
          .warning
          .as_ref()
          .and_then(|i| Some(warning_icon(i.into())))
          .unwrap_or("".into());
        let mutator = s
          .mutator
          .as_ref()
          .and_then(|i| Some(mutator_icon(i.into())))
          .unwrap_or("".into());
        format!(
          "Stage {}: {warning}{mutator}\n- {}\n- {}",
          s.id, s.primary, s.secondary
        )
      })
      .collect::<Vec<String>>()
      .join("\n");
    write!(f, "Seed: `{}`\n{stages}", self.seed)
  }
}
