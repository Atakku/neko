// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::plugins::drg::wrapper::get_deepdives;


commands! {
  fn drg(ctx) -> BasicCommand {
    let m = ctx.reply("Fetching Deep Dives...").await?;
    let res = get_deepdives().await?;
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
  }
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
