#![doc(html_root_url = "https://docs.rs/egui-dataframe-sample/0.3.3")]
//! egui dataframe sample
//!

use std::error::Error;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use eframe::{self, egui::*};
use egui_resources::{ResourcesBase, resized_copy_from};
use image::imageops::FilterType;
use polars::{series::Series, prelude::{ChunkApply}}; // , NamedFrom
use polars::prelude::{DataFrame, AnyValue, DataType}; // , Schema, Field
use egui_dataframe::{Decorator, DecoFs, DFDesc};
use anyvalue_dataframe::{row_schema, named_schema, df_from_vec, to_any};
use sqlite;
use polars_sqlite::{ToSqlite3ValueVec, IntoAnyValueVec};
// use polars_sqlite::{df_from_sl3, df_from_sl3_type};
// use polars_sqlite::{sl3_cols, sl3_tags, sl3_insert};

use itertools::Itertools;
use iter_tuple::{struct_derive, tuple_sqlite3, tuple_derive};

/// auto defines struct StTpl and sqlite3 trait with struct_derive
#[struct_derive((Rust, polars, sl3, RD, WR), (Utf8, Utf8, Utf8, Utf8, Utf8))]
/// auto defines sqlite3 trait for RecTpl with tuple_sqlite3
#[tuple_sqlite3(Utf8, Utf8, Utf8, Utf8, Utf8)]
/// auto defines struct RecTpl with tuple_derive
#[tuple_derive(Utf8, Utf8, Utf8, Utf8, Utf8)]
pub type Convtbl<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str);

/// auto defines struct StTpl and sqlite3 trait with struct_derive
#[struct_derive((ci64, ci8, cu64, cu8, cf64, cf32, cs, cb, cu),
  (Int64, Int8, UInt64, UInt8, Float64, Float32, Utf8, Boolean, Binary))]
/// auto defines sqlite3 trait for RecTpl with tuple_sqlite3
#[tuple_sqlite3(Int64, Int8, UInt64, UInt8, Float64, Float32, Utf8, Boolean, Binary)]
/// auto defines struct RecTpl with tuple_derive
#[tuple_derive(Int64, Int8, UInt64, UInt8, Float64, Float32, Utf8, Boolean, Binary)]
pub type Testtbl<'a> = (i64, i8, u64, u8, f64, f32, &'a str, bool, Vec<u8>);

/// auto defines struct StTpl and sqlite3 trait with struct_derive
#[struct_derive((n, str, s, id, b), (Int64, Utf8, Utf8, UInt64, Boolean))]
/// auto defines sqlite3 trait for RecTpl with tuple_sqlite3
#[tuple_sqlite3(Int64, Utf8, Utf8, UInt64, Boolean)]
/// auto defines struct RecTpl with tuple_derive
#[tuple_derive(Int64, Utf8, Utf8, UInt64, Boolean)]
pub type Tpl<'a> = (i64, &'a str, &'a str, u64, bool);

/// EguiDataFrameSample
// #[derive(Default)]
pub struct EguiDataFrameSample {
  /// start time
  pub start_time: Instant,
  /// counter
  pub cnt: u64,
  /// phase
  pub phase: u64,
  /// dataframe
  pub df: DataFrame,
  /// dfdesc: DFDesc
  pub dfdesc: DFDesc,
  /// large deco
  pub ld: Vec<Decorator>,
  /// image
  pub img: ColorImage,
  /// base path of resources
  pub bp: ResourcesBase
}

/// EguiDataFrameSample
impl EguiDataFrameSample {
  /// constructor
  pub fn new(cc: &eframe::CreationContext<'_>, bp: ResourcesBase) -> Self {
    let ffs = vec![
      ("FiraSans", "FiraSans-Regular.ttf", FontFamily::Monospace),
      ("FiraSansP", "FiraSans-Regular.ttf", FontFamily::Proportional)];
    cc.egui_ctx.set_fonts(bp.reg_fonts(ffs));

    let rows = [
      (0, "Alpha", "a", 0, true),
      (12345, "Bravo", "b", 1, false),
      (2468, "Charlie", "c", 2, false),
      (1357, "Delta", "d", 3, true),
      (-1234, "Epsilon", "e", 4, true)
    ].into_iter().map(|r|
      row_schema(RecTpl::into_iter(r).collect())
//    row_schema(RecTpl::from(r).into_iter().collect())
//    row_schema(RecTpl::from(r).v)
    ).collect::<Vec<_>>();

    let n = StTpl::members();
/*
    let schema = Schema::from(&rows[0]);
    let df = DataFrame::from_rows_iter_and_schema(rows.iter(), &schema);
    let mut df = df.expect("create DataFrame");
    df.set_column_names(&n).expect("set column names");
*/
    let df = df_from_vec(&rows, &n).expect("create DataFrame");
    let sc = named_schema(&df, n);

/*
    let df = df.select(["b", "id", "s", "str", "n"]).expect("select columns");
    println!("{:?}", df.head(Some(100)));
*/

//    let sl3name = "test_sqlite_write.sl3";
//    let dbn = bp.basepath.join(sl3name).to_str().expect("utf8");
//&sl3_cols(&n, (true, 3));
//&sl3_tags(&n, (true, 3));

    let deco = Decorator::new(Vec2::new(50.0, 16.0), Sense::hover(), vec![],
      Align2::LEFT_TOP, Vec2::new(2.0, 0.0), FontId::proportional(9.0));
    let decos = [
      [Color32::BROWN, Color32::YELLOW, Color32::BLUE],
      [Color32::YELLOW, Color32::BROWN, Color32::GREEN]
    ].iter().map(|v| {
      let mut d = deco.clone(); d.cols = Decorator::opt(v); d
    }).collect_tuple().expect("Decorator tuple");
    let dfdesc = DFDesc::new(decos, sc).all_default();

    let sz = Vec2::new(320.0 - 16.0 - 16.0, 32.0); // - margin size - img size
    let sense = Sense::hover();
    let cols = [
      [Color32::YELLOW, Color32::GREEN, Color32::BLUE],
      [Color32::BROWN, Color32::YELLOW, Color32::RED]];
    let align = Align2::LEFT_TOP;
    let ofs = Vec2::new(2.0, -4.0);
    let fontid = FontId::proportional(24.0);
    let ld = cols.iter().map(|v|
      Decorator::new(sz, sense, Decorator::opt(v), align, ofs, fontid.clone())
    ).collect::<Vec<_>>();

    let img = bp.resource_img("_4c_4x4.png", true);
    let img = resized_copy_from([16, 16], &img, FilterType::Lanczos3);

    EguiDataFrameSample{start_time: Instant::now(), cnt: 0, phase: 0,
      df, dfdesc, ld, img, bp}
  }
}

/// eframe::App for EguiDataFrameSample
impl eframe::App for EguiDataFrameSample {
  fn update(&mut self, ctx: &Context, frm: &mut eframe::Frame) {
    let elapsed_time = self.start_time.elapsed().as_millis();
    if elapsed_time >= Duration::from_millis(1000).as_millis() {
      self.cnt += 1;
      self.start_time = Instant::now();
    }

    let evts = ctx.input(|is| is.raw.clone()).events;
    for ev in evts.iter() {
      match ev {
      Event::Key{key: Key::C, pressed, modifiers: Modifiers{
        alt: _, ctrl: true, shift: _, mac_cmd: _, command: _}, repeat: _} => {
        if pressed == &true { frm.close(); }
      },
      _ => {},
      }
    }

    let c = [Color32::GREEN, Color32::RED, Color32::YELLOW];
    let mut f = DecoFs{fncs: (
      &mut DecoFs::default,
      &mut |d: &Decorator, ui, tx, ri, ci| {
        let t = format!("{} {} {}", ri, ci, tx);
        let mut d = d.clone();
        if ri == 2 || ci == 1 { d.cols = Decorator::opt(&c); }
        d.disp(ui, &t);
        true
      }
    )};

    let _pl = SidePanel::left("left").show(ctx, |ui| {
      ui.label(RichText::new("Left").size(32.0));
      let _r_p = self.ld[0].disp(ui, "Left"); // Some((resp, painter)) or None
      self.dfdesc.disp(ui, &mut f, &self.df, 5.0, 12.0, true, true, true);
    });
    let _pr = SidePanel::right("right").show(ctx, |ui| {
      ui.label(RichText::new("Right").size(32.0));
      let _r_p = self.ld[1].disp(ui, "Right"); // Some((resp, painter)) or None
      self.dfdesc.grid(ui, &mut f, &self.df, 50.0, 18.0, &TextStyle::Small,
        &(1.0, 1.0), &style::Margin::same(1.0));
    });
    CentralPanel::default().show(ctx, |ui| {
      ui.label(RichText::new("Center").size(32.0));
      ScrollArea::both().show(ui, |ui| {
        let img = self.img.clone();
        let tex = ui.ctx().load_texture("img", img, Default::default());
        ui.add(Image::new(&tex, tex.size_vec2()))
      });
    });

    // let columns = self.df.get_columns_mut();
    // columns[3].u64().expect("id series as u64")[2] = self.cnt; // not index
    let columns = self.df.get_columns();
    let column_3 = columns[3].u64().expect("id series as u64"); // ChunkedArray
    let series_id = Series::from(column_3.apply_with_idx(|(i, s)|
      if i == 2 { self.cnt } else { s + 1 }));
    self.df.replace("id", series_id).expect("replace df is_ok");

    ctx.request_repaint_after(Duration::from_millis(17));
  }
}

/// main
pub fn main() -> Result<(), Box<dyn Error>> {
  let bp = ResourcesBase::new(PathBuf::from("./resources"));
  let opts = eframe::NativeOptions{
    initial_window_size: Some((640.0, 480.0).into()),
    initial_window_pos: Some(Pos2{x: 0.0, y: 0.0}), // or parsistence true
    resizable: false,
    icon_data: bp.resource_icon("_4c_4x4.png", true),
//    default_theme: Theme::Light,
    ..eframe::NativeOptions::default()
  };
  eframe::run_native("egui dataframe sample", opts,
    Box::new(|cc| Box::new(EguiDataFrameSample::new(cc, bp))))?;
  Ok(())
}
