// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use askama::Template;
use image::EncodableLayout;
use resvg::{
  tiny_skia::Pixmap,
  usvg::{fontdb::Database, Options, Transform, Tree, TreeParsing, TreeTextToPath},
};

once_cell!(fontdb, FONTDB: Database);

module!(
  SvgUi {}

  fn init(fw) {
    FONTDB.set({
      let mut fontdb = Database::new();
      fontdb.load_system_fonts();
      fontdb.load_font_file("font.ttf").unwrap();
      fontdb
    })?;

    runtime!(fw, |m| {
      Ok(None)
    });
  }
);

const SCALEUP: f32 = 1.0;

pub async fn render_svg(template: impl Template) -> Res<Vec<u8>> {
  let mut tree = Tree::from_str(&template.render()?, &Options::default())?;
  tree.convert_text(fontdb());
  let mut pixmap = Pixmap::new(
    (tree.size.width() / SCALEUP).round() as u32,
    (tree.size.height() / SCALEUP).round() as u32,
  )
  .ok_or::<Err>("Failed to create pixmap".into())?;
  let retree = resvg::Tree::from_usvg(&tree);
  retree.render(
    Transform::from_scale(1.0/SCALEUP, 1.0/SCALEUP),
    &mut pixmap.as_mut(),
  );
  let encoder = webp::Encoder::new(
    pixmap.data(),
    webp::PixelLayout::Rgba,
    pixmap.width(),
    pixmap.height(),
  );
  let data = encoder
    .encode_simple(false, 80.0)
    .map_err(|_| "Failed to encode webp")?;
  Ok(data.as_bytes().to_vec())
}
