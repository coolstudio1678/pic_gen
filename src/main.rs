use chrono::prelude::*;
use egui::*;
use egui_plot::{
    Bar, BarChart, BoxElem, BoxPlot, BoxSpread, Corner, Legend, Line, Plot, PlotPoints,
};
use polars::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Shuaixuan {
    vec_code: Vec<String>,
    vec_date: Vec<String>,
    iindex: i32,
    code: String,
    date: String,
    saving:bool,
    started:bool,
    // df_ohlc:Option<DataFrame>,
}

impl Default for Shuaixuan {
    fn default() -> Self {
        let mut schema = Schema::default();
        schema.with_column("code".into(), DataType::String);
        schema.with_column("date".into(), DataType::String);

        let file = "510_8new1.csv";
        let df_code = CsvReadOptions::default()
            .with_has_header(true)
            .with_schema(Some(Arc::new(schema)))
            .try_into_reader_with_file_path(Some(file.into()))
            .unwrap()
            .finish()
            .unwrap();

        let vec_code: Vec<String> = df_code
            .column("code")
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .map(|x| x.map(|x| x.to_string()).unwrap())
            .collect::<Vec<_>>();
        let vec_date: Vec<String> = df_code
            .column("date")
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .map(|x| x.map(|x| x.to_string()).unwrap())
            .collect::<Vec<_>>();

        let current_dir = std::env::current_dir().expect("无法获取当前目录");
        let file_path = current_dir.join("index.csv");
        dbg!(&file_path);
        let mut file = std::fs::File::open(file_path).expect("file not found");
        let mut content = String::new();
        std::io::Read::read_to_string(&mut file, &mut content).expect("0");
        // let iindex = content.parse::<i32>().unwrap();
        let iindex = 0;
        let code = vec_code[0].to_string();
        let date = vec_date[0].to_string();

        Self {
            vec_code,
            vec_date,
            iindex: iindex,
            code,
            date,
            saving:false,
            started:false,
            // df_ohlc:None,
        }
    }
}
impl Shuaixuan {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            ..Default::default()
        }
    }

    pub fn normalize(data: &Vec<f64>) -> Vec<f64> {
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if (max - min).abs() < f64::EPSILON {
            return vec![0.0; data.len()];
        }
        data.iter().map(|x| (x - min) / (max - min)).collect()
    }

    fn get_plot_data(&mut self) -> PlotData {
        let file_posi = "d:/data/day/".to_string();
        let file_hour = "d:/data/hour/".to_string();

        let mut schema = Schema::default();
        schema.with_column("code".into(), DataType::String);
        schema.with_column("date".into(), DataType::String);
        schema.with_column("open".into(), DataType::Float64);
        schema.with_column("high".into(), DataType::Float64);
        schema.with_column("low".into(), DataType::Float64);
        schema.with_column("close".into(), DataType::Float64);
        schema.with_column("volume".into(), DataType::Int64);

        let mut schema1 = Schema::default();
        schema1.with_column("code".into(), DataType::String);
        schema1.with_column("time".into(), DataType::String);
        schema1.with_column("open".into(), DataType::Float64);
        schema1.with_column("high".into(), DataType::Float64);
        schema1.with_column("low".into(), DataType::Float64);
        schema1.with_column("close".into(), DataType::Float64);
        schema1.with_column("volume".into(), DataType::Int64);

        let iindex = self.iindex;
        let code = &self.vec_code[iindex as usize];
        // if !code == self.code {s}
        let date = &self.vec_date[iindex as usize];
        self.code = code.clone();
        self.date = date.clone();

        let ff = format!("{}{}.csv", file_posi.clone(), code.to_string());
        let ff_hour = format!("{}{}.csv", file_hour.clone(), code.to_string());

        let mut df_day = CsvReadOptions::default()
            .with_has_header(true)
            .with_schema_overwrite(Some(Arc::new(schema)))
            .try_into_reader_with_file_path(Some(ff.into()))
            .unwrap()
            .finish()
            .unwrap();
        df_day = df_day
            .lazy()
            .with_columns([col("date").cast(DataType::Date), lit("1").alias("index")])
            .with_columns([col("index").cum_count(false).alias("iindex")])
            .with_columns([
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 5,
                        ..Default::default()
                    })
                    .alias("ma5"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 10,
                        ..Default::default()
                    })
                    .alias("ma10"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 20,
                        ..Default::default()
                    })
                    .alias("ma20"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 30,
                        ..Default::default()
                    })
                    .alias("ma30"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 60,
                        ..Default::default()
                    })
                    .alias("ma60"),
                col("close")
                    .ewm_mean(EWMOptions {
                        alpha: (2.0 / 13.0),
                        ..Default::default()
                    })
                    .alias("short"),
                col("close")
                    .ewm_mean(EWMOptions {
                        alpha: (2.0 / 27.0),
                        ..Default::default()
                    })
                    .alias("long"),
            ])
            .with_columns([(col("short") - col("long")).alias("diff")])
            .with_columns([col("diff")
                .ewm_mean(EWMOptions {
                    alpha: (2.0 / 10.0),
                    ..Default::default()
                })
                .alias("dea")])
            .with_columns([((col("diff") - col("dea")) * lit(2.0)).alias("macd")])
            .collect()
            .unwrap();

        let mut df_hour = CsvReadOptions::default()
            .with_has_header(true)
            .with_schema_overwrite(Some(Arc::new(schema1)))
            .try_into_reader_with_file_path(Some(ff_hour.into()))
            .unwrap()
            .finish()
            .unwrap();
        df_hour = df_hour
            .lazy()
            .with_columns([
                col("time").str().strptime(
                    DataType::Datetime(TimeUnit::Nanoseconds, None),
                    StrptimeOptions {
                        format: Some(PlSmallStr::from_str("%Y%m%d%H%M%S%3f")),
                        strict: false,
                        ..Default::default()
                    },
                    lit("raise"),
                ),
                lit(1).alias("index"),
            ])
            .with_columns([
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 5,
                        ..Default::default()
                    })
                    .alias("ma5"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 10,
                        ..Default::default()
                    })
                    .alias("ma10"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 20,
                        ..Default::default()
                    })
                    .alias("ma20"),
                col("close")
                    .rolling_mean(RollingOptionsFixedWindow {
                        window_size: 30,
                        ..Default::default()
                    })
                    .alias("ma30"),
                // (((col("close") - col("close").shift(1.into())) / col("close").shift(1.into())) * lit(100.0)).alias("zdf"),
                col("close")
                    .ewm_mean(EWMOptions {
                        alpha: (2.0 / 13.0),
                        ..Default::default()
                    })
                    .alias("short"),
                col("close")
                    .ewm_mean(EWMOptions {
                        alpha: (2.0 / 27.0),
                        ..Default::default()
                    })
                    .alias("long"),
            ])
            .with_columns([(col("short") - col("long")).alias("diff")])
            .with_columns([col("diff")
                .ewm_mean(EWMOptions {
                    alpha: (2.0 / 10.0),
                    ..Default::default()
                })
                .alias("dea")])
            .with_columns([((col("diff") - col("dea")) * lit(2.0)).alias("macd")])
            .collect()
            .unwrap();

        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let next_date = date.succ_opt().unwrap();
        let mut select_df = df_day
            .clone()
            .lazy()
            .filter(col("date").lt_eq(lit(date)))
            .collect()
            .unwrap();
        let select_hour = df_hour
            .clone()
            .lazy()
            .filter(col("time").lt(lit(next_date)))
            .collect()
            .unwrap()
            .tail(Some(60));

        // println!("index: {}", self.iindex);

        select_df = select_df.tail(Some(50));
        let lens = select_df.height();

        let mut box_elems: Vec<BoxElem> = Vec::new();
        let mut line_ma5: Vec<Vec<f64>> = Vec::new();
        let mut line_ma10: Vec<Vec<f64>> = Vec::new();
        let mut line_ma20: Vec<Vec<f64>> = Vec::new();
        let mut line_ma30: Vec<Vec<f64>> = Vec::new();
        let mut line_ma60: Vec<Vec<f64>> = Vec::new();
        let mut diff: Vec<Vec<f64>> = Vec::new();
        let mut dea: Vec<Vec<f64>> = Vec::new();
        let mut macd: Vec<Bar> = Vec::new();
        let mut vol_day: Vec<Bar> = Vec::new();

        let colorred = Color32::from_rgb(255, 0, 0);
        let colorgreen = Color32::from_rgb(0, 255, 0);
        let colorwhite = Color32::from_rgb(255, 255, 255);

        let volume = select_df
            .tail(Some(50))
            .clone()
            .column("volume")
            .unwrap()
            .i64()
            .unwrap()
            .iter()
            .map(|x| x.unwrap_or_else(|| 0) as f64)
            .collect();
        let volumes = Shuaixuan::normalize(&volume);
        for i in 1..lens {
            let open = select_df
                .column("open")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let close = select_df
                .column("close")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let high = select_df
                .column("high")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let low = select_df
                .column("low")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma5 = select_df
                .column("ma5")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma10 = select_df
                .column("ma10")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma20 = select_df
                .column("ma20")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma30 = select_df
                .column("ma30")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma60 = select_df
                .column("ma60")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let diff_one = select_df
                .column("diff")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let dea_one = select_df
                .column("dea")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let macd_one = select_df
                .column("macd")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();

            let volume = volumes[i];

            let oo = if open < close { open } else { close };
            let cc = if open < close { close } else { open };
            let color1 = if open < close {
                colorred
            } else if open > close {
                colorgreen
            } else {
                colorwhite
            };
            let box1 = BoxElem::new(i as f64, BoxSpread::new(low, oo, oo, cc, high))
                .whisker_width(0.0)
                .fill(color1)
                .stroke(Stroke::new(1.6, color1));

            let box2 = Bar::new(i as f64, volume).fill(color1);
            let b_macd = Bar::new(i as f64, macd_one);

            box_elems.push(box1);
            let line5 = [i as f64, ma5];
            let line10 = [i as f64, ma10];
            let line20 = [i as f64, ma20];
            let line30 = [i as f64, ma30];
            let line60 = [i as f64, ma60];
            let t_diff = [i as f64, diff_one];
            let t_dea = [i as f64, dea_one];

            line_ma5.push(line5.to_vec());
            line_ma10.push(line10.to_vec());
            line_ma20.push(line20.to_vec());
            line_ma30.push(line30.to_vec());
            line_ma60.push(line60.to_vec());
            diff.push(t_diff.to_vec());
            dea.push(t_dea.to_vec());
            macd.push(b_macd);
            vol_day.push(box2);
        }

        let lens = select_hour.height();
        let mut box_elems_hour: Vec<BoxElem> = Vec::new();
        let mut hour_ma5: Vec<Vec<f64>> = Vec::new();
        let mut hour_ma10: Vec<Vec<f64>> = Vec::new();
        let mut hour_ma20: Vec<Vec<f64>> = Vec::new();
        let mut hour_ma30: Vec<Vec<f64>> = Vec::new();
        let mut hour_diff: Vec<Vec<f64>> = Vec::new();
        let mut hour_dea: Vec<Vec<f64>> = Vec::new();
        let mut hour_macd: Vec<Bar> = Vec::new();
        let mut vol_hour: Vec<Bar> = Vec::new();

        let volume = select_hour
            .clone()
            .column("volume")
            .unwrap()
            .i64()
            .unwrap()
            .iter()
            .map(|x| x.unwrap_or_else(|| 0) as f64)
            .collect();
        let volumes = Shuaixuan::normalize(&volume);

        for i in 1..lens {
            let open = select_hour
                .column("open")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let close = select_hour
                .column("close")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let high = select_hour
                .column("high")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let low = select_hour
                .column("low")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma5 = select_hour
                .column("ma5")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma10 = select_hour
                .column("ma10")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma20 = select_hour
                .column("ma20")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let ma30 = select_hour
                .column("ma30")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let diff_one = select_hour
                .column("diff")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let dea_one = select_hour
                .column("dea")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let macd_one = select_hour
                .column("macd")
                .unwrap()
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap();
            let volume_h = volumes[i];

            let oo = if open < close { open } else { close };
            let cc = if open < close { close } else { open };
            let color1 = if open < close {
                colorred
            } else if open > close {
                colorgreen
            } else {
                colorwhite
            };
            let box1 = BoxElem::new(i as f64, BoxSpread::new(low, oo, oo, cc, high))
                .whisker_width(0.0)
                .fill(color1)
                .stroke(Stroke::new(0.8, color1));
            let box2 = Bar::new(i as f64, volume_h).fill(color1);
            let b_macd = Bar::new(i as f64, macd_one);

            box_elems_hour.push(box1);
            let line5 = [i as f64, ma5];
            let line10 = [i as f64, ma10];
            let line20 = [i as f64, ma20];
            let line30 = [i as f64, ma30];
            let t_diff = [i as f64, diff_one];
            let t_dea = [i as f64, dea_one];
            hour_ma5.push(line5.to_vec());
            hour_ma10.push(line10.to_vec());
            hour_ma20.push(line20.to_vec());
            hour_ma30.push(line30.to_vec());
            hour_diff.push(t_diff.to_vec());
            hour_dea.push(t_dea.to_vec());
            hour_macd.push(b_macd);
            vol_hour.push(box2);
        }

        // 创建日线数据结构
        let day_data = DayPlotData {
            box_elems,
            line_ma5,
            line_ma10,
            line_ma20,
            line_ma30,
            line_ma60,
            diff,
            dea,
            macd,
            vol_day,
        };

        // 创建小时线数据结构
        let hour_data = HourPlotData {
            box_elems: box_elems_hour,
            line_ma5: hour_ma5,
            line_ma10: hour_ma10,
            line_ma20: hour_ma20,
            line_ma30: hour_ma30,
            hour_diff,
            hour_dea,
            hour_macd,
            vol_hour,
        };

        // 返回组合的数据结构
        PlotData {
            day: day_data,
            hour: hour_data,
        }
    }
}

#[derive(Debug, Clone)]
struct DayPlotData {
    box_elems: Vec<BoxElem>,
    line_ma5: Vec<Vec<f64>>,
    line_ma10: Vec<Vec<f64>>,
    line_ma20: Vec<Vec<f64>>,
    line_ma30: Vec<Vec<f64>>,
    line_ma60: Vec<Vec<f64>>,
    diff: Vec<Vec<f64>>,
    dea: Vec<Vec<f64>>,
    macd: Vec<Bar>,
    vol_day: Vec<Bar>,
}

#[derive(Debug, Clone)]
struct HourPlotData {
    box_elems: Vec<BoxElem>,
    line_ma5: Vec<Vec<f64>>,
    line_ma10: Vec<Vec<f64>>,
    line_ma20: Vec<Vec<f64>>,
    line_ma30: Vec<Vec<f64>>,
    hour_diff: Vec<Vec<f64>>,
    hour_dea: Vec<Vec<f64>>,
    hour_macd: Vec<Bar>,
    vol_hour: Vec<Bar>,
}

#[derive(Debug, Clone)]
struct PlotData {
    day: DayPlotData,
    hour: HourPlotData,
}

impl eframe::App for Shuaixuan {
    // 保存数据
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        let _s_code = Series::new(PlSmallStr::from_str("code"), self.vec_code.clone());
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        if self.iindex > 77601 {  //77601
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        eframe::egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.label(self.code.clone());
            ui.label(self.date.clone());
            ui.separator();
            ui.add_space(12.0);

            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                //ArrowLeft,   ArrowRight,
                let file_path = Path::new("index.txt");
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(file_path)
                    .unwrap();
                let content = self.iindex.to_string();
                file.write_all(content.as_bytes()).unwrap();

                self.iindex = self.iindex + 1;
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                //ArrowLeft,   ArrowRight,
                self.iindex = self.iindex - 1;
            }

            if ui
                .add_sized([ui.available_width(), 20.0], egui::Button::new("next"))
                .clicked()
            {
                // 保存数据

                let file_path = Path::new("index.txt");
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(file_path)
                    .unwrap();
                let content = self.iindex.to_string();
                file.write_all(content.as_bytes()).unwrap();
                self.iindex = self.iindex + 1;
            }
            if ui
                .add_sized([ui.available_width(), 20.0], egui::Button::new("preview"))
                .clicked()
            {
                self.iindex = self.iindex - 1;
            }
            ui.add_space(10.0);
            if ui
                .add_sized(
                    [ui.available_width(), 20.0],
                    egui::Button::new("Save as Samples"),
                )
                .clicked()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }

            if ui
                .add_sized(
                    [ui.available_width(), 20.0],
                    egui::Button::new("Started"),
                )
                .clicked()
            {
                self.started = true;
            }

            ui.add_space(20.0);
            ui.label(format!("index:{}", self.iindex));

            // ui.input(|i| {
            //     for event in &i.raw.events {
            //         if let egui::Event::Screenshot { image, .. } = event {
            //             let pixels_per_point = i.pixels_per_point();
            //             let region = egui::Rect::from_two_pos(
            //                 egui::Pos2::ZERO, //{x:0., y:60.},
            //                 egui::Pos2 { x: 800., y: 930. },
            //             );
            //             let top_left_corner = image.region(&region, Some(pixels_per_point));
            //             let out_name = format!("{}-{}.png",self.code,self.date);
            //             image::save_buffer(
            //                 out_name,
            //                 top_left_corner.as_raw(),
            //                 top_left_corner.width() as u32,
            //                 top_left_corner.height() as u32,
            //                 image::ColorType::Rgba8,
            //             )
            //             .unwrap();
            //         }
            //     }
            // });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_data = self.get_plot_data();

            // 日线部分 - 使用垂直布局
            ui.vertical(|ui| {
                // K线图
                let _legend = Legend::default().position(Corner::LeftTop);
                let plot = Plot::new("Candlestick day")
                    // .legend(legend.clone())
                    .allow_zoom(false)
                    .allow_drag(false)
                    .view_aspect(3.0)
                    .show_grid(false)
                    .show_y(false);

                let boxplot = BoxPlot::new(plot_data.day.box_elems).name("OHLC");

                let ma5: PlotPoints = plot_data
                    .day
                    .line_ma5
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema5 = Line::new(ma5)
                    .color(Color32::from_rgb(255, 0, 0))
                    .width(0.8)
                    .name("ma5_");
                let ma10: PlotPoints = plot_data
                    .day
                    .line_ma10
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema10 = Line::new(ma10)
                    .color(Color32::from_rgb(0, 255, 0))
                    .width(0.8)
                    .name("ma10");
                let ma20: PlotPoints = plot_data
                    .day
                    .line_ma20
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema20 = Line::new(ma20)
                    .color(Color32::from_rgb(0, 0, 255))
                    .width(0.8)
                    .name("ma20");
                let ma30: PlotPoints = plot_data
                    .day
                    .line_ma30
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema30 = Line::new(ma30)
                    .color(Color32::from_rgb(125, 100, 125))
                    .width(0.8)
                    .name("ma30");
                let ma60: PlotPoints = plot_data
                    .day
                    .line_ma60
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema60 = Line::new(ma60)
                    .color(Color32::from_rgb(255, 200, 200))
                    .width(0.8)
                    .name("ma60");
                let diff: PlotPoints = plot_data.day.diff.iter().map(|x| [x[0], x[1]]).collect();
                let line_diff = Line::new(diff)
                    .color(Color32::from_rgb(255, 255, 255))
                    .width(0.8);

                let dea: PlotPoints = plot_data.day.dea.iter().map(|x| [x[0], x[1]]).collect();
                let line_dea = Line::new(dea)
                    .color(Color32::from_rgb(255, 255, 0))
                    .width(0.8);
                let b_macd = BarChart::new(plot_data.day.macd).width(0.05);

                let chart = BarChart::new(plot_data.day.vol_day);
                // .width(1.5);

                plot.show(ui, |plot_ui| {
                    plot_ui.box_plot(boxplot);
                    plot_ui.line(linema5);
                    plot_ui.line(linema10);
                    plot_ui.line(linema20);
                    plot_ui.line(linema30);
                    plot_ui.line(linema60);
                });

                let macd_plot = Plot::new("Day Macd")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .view_aspect(8.0) // 成交量图高度较小
                    
                    .show_grid(false);

                macd_plot.show(ui, |plot_ui| {
                    plot_ui.line(line_diff);
                    plot_ui.line(line_dea);
                    plot_ui.bar_chart(b_macd);
                });

                // 日线成交量图
                let volume_plot = Plot::new("Day Volume")
                    .allow_zoom(false)
                    .allow_drag(false)
                    // .y_axis_label("vol")
                    .view_aspect(8.0) // 成交量图高度较小
                    .show_grid(false);

                volume_plot.show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);
                });
            });

            // ui.separator();

            // 小时线部分 - 使用垂直布局
            ui.vertical(|ui| {
                // 小时K线图
                let _legend = Legend::default().position(Corner::LeftTop);
                let plot_hour = Plot::new("Candlestick hour")
                    // .legend(legend)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .view_aspect(3.0)
                    .show_grid(false)
                    .show_y(false);

                let boxplot = BoxPlot::new(plot_data.hour.box_elems).name("OHLC");
                let ma5: PlotPoints = plot_data
                    .hour
                    .line_ma5
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema5 = Line::new(ma5)
                    .color(Color32::from_rgb(255, 0, 0))
                    .width(0.8)
                    .name("ma5");
                let ma10: PlotPoints = plot_data
                    .hour
                    .line_ma10
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema10 = Line::new(ma10)
                    .color(Color32::from_rgb(0, 255, 0))
                    .width(0.8)
                    .name("ma10");
                let ma20: PlotPoints = plot_data
                    .hour
                    .line_ma20
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema20 = Line::new(ma20)
                    .color(Color32::from_rgb(0, 0, 255))
                    .width(0.8)
                    .name("ma20");
                let ma30: PlotPoints = plot_data
                    .hour
                    .line_ma30
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let linema30 = Line::new(ma30)
                    .color(Color32::from_rgb(125, 100, 125))
                    .width(0.8)
                    .name("ma30");

                let diff: PlotPoints = plot_data
                    .hour
                    .hour_diff
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let line_diff = Line::new(diff)
                    .color(Color32::from_rgb(255, 255, 255))
                    .width(0.8);

                let dea: PlotPoints = plot_data
                    .hour
                    .hour_dea
                    .iter()
                    .map(|x| [x[0], x[1]])
                    .collect();
                let line_dea = Line::new(dea)
                    .color(Color32::from_rgb(255, 255, 0))
                    .width(0.8);
                let b_macd = BarChart::new(plot_data.hour.hour_macd).width(0.1);
                let chart = BarChart::new(plot_data.hour.vol_hour);

                // .width(1.5);

                plot_hour.show(ui, |plot_ui| {
                    plot_ui.box_plot(boxplot);
                    plot_ui.line(linema5);
                    plot_ui.line(linema10);
                    plot_ui.line(linema20);
                    plot_ui.line(linema30);
                });
                let macd_plot = Plot::new("Hour Macd")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .view_aspect(8.0) // 成交量图高度较小
                    .show_grid(false);

                macd_plot.show(ui, |plot_ui| {
                    plot_ui.line(line_diff);
                    plot_ui.line(line_dea);
                    plot_ui.bar_chart(b_macd);
                });

                // 小时成交量图
                let hour_volume_plot = Plot::new("Hour Volume")
                    .allow_zoom(false)
                    .allow_drag(false)
                    // .y_axis_label("vol")
                    .view_aspect(8.0) // 成交量图高度较小
                    .show_grid(false);

                hour_volume_plot.show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);
                });
            });
        });

        if self.started{
            if !self.saving {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.saving = true;        
            }
        }
        ctx.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    let pixels_per_point = i.pixels_per_point();
                    let region = egui::Rect::from_two_pos(
                        egui::Pos2::ZERO, //{x:0., y:60.},
                        egui::Pos2 { x: 800., y: 930. },
                    );
                    let top_left_corner = image.region(&region, Some(pixels_per_point));
                    let out_name = format!("./pic/{}_{}.png",self.code,self.date);
                    image::save_buffer(
                        out_name,
                        top_left_corner.as_raw(),
                        top_left_corner.width() as u32,
                        top_left_corner.height() as u32,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                    self.iindex +=1;
                    println!("{}",self.iindex);
                    self.saving = false; 
                }
            }
        });


        
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        // resizable: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 980.0])
            .with_min_inner_size([1000.0, 980.0]) // 新增这一行
            .with_max_inner_size([1000.0, 980.0]), // 新增这一行,            
        ..Default::default()
    };
    // let current_dir = env::current_dir().unwrap();
    // println!("Current directory: {}", current_dir.display());

    let _ret = eframe::run_native(
        "My app",
        native_options,
        Box::new(|cc| Ok(Box::new(Shuaixuan::new(cc)))),
    );
}
