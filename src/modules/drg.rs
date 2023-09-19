// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  interface::drg::{Variant, DeepRockGalacticApi},
  modules::{
    poise::{Ctx, Poise},
  },
};
use std::fmt;

pub struct DeepRockGalactic;

impl Module for DeepRockGalactic {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(drg());
    Ok(())
  }
}
#[poise::command(prefix_command, slash_command)]
pub async fn drg(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Fetching Deep Dives...").await?;
  let res = reqwest!().get_deepdives().await?;
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
    "Cave Leech Cluster" => "<:warning_cave_leech_cluster:1152827797476233266>",
    "Elite Threat" => "<:warning_elite_threat:1152827795039342622>",
    "Exploder Infestation" => "<:warning_exploder_infestation:1152827792636002375>",
    "Haunted Cave" => "<:warning_haunted_cave:1152827790761136198>",
    "Lethal Enemies" => "<:warning_lethal_enemies:1152827787493769286>",
    "Low Oxygen" => "<:warning_low_oxygen:1152827817722118276>",
    "Mactera Plague" => "<:warning_mactera_plague:1152827814538645618>",
    "Parasites" => "<:warning_parasites:1152827811933978756>",
    "Regenerative Bugs" => "<:warning_regenerative_bugs:1152827806976311437>",
    "Rival Presence" => "<:warning_rival_presence:1152827803121762356>",
    "Shield Disruption" => "<:warning_shield_disruption:1152827801330798592>",
    "Swarmageddon" => "<:warning_swarmageddon:1152827799715975198>",
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
