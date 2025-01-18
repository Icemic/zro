use std::collections::BinaryHeap;

use egui::epaint::text::{FontInsert, InsertFontFamily};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Row {
    pub sim: f64,
    pub key: String,
    pub value: String,
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.sim == other.sim
    }
}

impl Eq for Row {}

impl PartialOrd for Row {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.sim.partial_cmp(&other.sim)
    }
}

impl Ord for Row {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.sim > other.sim {
            std::cmp::Ordering::Greater
        } else if self.sim < other.sim {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
        // other.sim.cmp(&self.sim)
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    datasource: Vec<(String, String)>,

    is_always_on_top: bool,
    keywords: String,
    results: Vec<Row>,
}

#[derive(PartialEq, Deserialize, Serialize)]
enum State {
    Loading,
    Ready,
    Fatal(String),
    Error(String),
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: State::Loading,
            datasource: vec![],
            is_always_on_top: false,
            keywords: "".to_owned(),
            results: vec![],
        }
    }
}

const DATA: &[u8] = include_bytes!("../data.csv");

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        cc.egui_ctx.add_font(FontInsert::new(
            "SarasaUiSC-Regular",
            egui::FontData::from_static(include_bytes!("../assets/SarasaUiSC-Regular.ttf")),
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: egui::epaint::text::FontPriority::Highest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: egui::epaint::text::FontPriority::Lowest,
                },
            ],
        ));

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            if app.is_always_on_top {
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                        egui::WindowLevel::AlwaysOnTop,
                    ));
            }

            let external_data_path = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("data.csv");

            let data = if external_data_path.exists() {
                &std::fs::read(external_data_path).unwrap()
            } else {
                DATA
            };

            let mut csv = csv::Reader::from_reader(data);

            let mut records = csv.records();
            while let Some(record) = records.next() {
                match record {
                    Ok(record) => {
                        app.datasource.push((
                            record.get(0).unwrap().replace('_', " "),
                            record.get(1).unwrap().to_string(),
                        ));
                    }
                    Err(e) => {
                        app.state = State::Error(e.to_string());
                    }
                }
            }

            return app;
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    let always_on_top = ui.button("窗口置顶");
                    if always_on_top.clicked() {
                        self.is_always_on_top = !self.is_always_on_top;
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                            egui::WindowLevel::AlwaysOnTop,
                        ));
                    }
                    if self.is_always_on_top {
                        always_on_top.highlight();
                    }
                }

                egui::widgets::global_theme_preference_switch(ui);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("请到 ");
                    ui.hyperlink_to("Github", "https://github.com/Icemic/zro");
                    ui.label(" 反馈问题。");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state == State::Loading {
                ui.label("Loading...");
                self.state = State::Ready;
            } else if let State::Fatal(e) = &self.state {
                ui.label(format!("Fatal: {}", e));
            } else {
                if let State::Error(e) = &self.state {
                    ui.label(format!("Error: {}", e));
                }

                let editor = ui.text_edit_singleline(&mut self.keywords);

                if !editor.has_focus() {
                    editor.request_focus();
                }

                if editor.changed() {
                    let mut heap = BinaryHeap::new();
                    for (key, value) in &self.datasource {
                        let max_sim = if self.keywords.chars().count() <= 1 {
                            let sim_a =
                                strsim::normalized_damerau_levenshtein(&key, &self.keywords);
                            let sim_b =
                                strsim::normalized_damerau_levenshtein(&value, &self.keywords);
                            sim_a.max(sim_b)
                        } else {
                            let sim_a = strsim::sorensen_dice(&key, &self.keywords);
                            let sim_b = strsim::sorensen_dice(&value, &self.keywords);
                            sim_a.max(sim_b)
                        };

                        heap.push(Row {
                            sim: max_sim,
                            key: key.clone(),
                            value: value.clone(),
                        });
                    }
                    self.results.clear();
                    for _ in 0..20 {
                        if let Some(row) = heap.pop() {
                            self.results.push(row);
                        }
                    }
                }

                ui.separator();

                ui.label("点击对应条目行复制提示词");

                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .max_col_width(240.0)
                    .show(ui, |ui| {
                        // ui.label("相似度");
                        ui.label("关键词").clicked();
                        ui.label("内容");
                        ui.end_row();
                        for (_, row) in self.results.iter().enumerate() {
                            // ui.label(format!("{:.2}", row.sim));
                            if ui
                                .label(format!("{}", row.key))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                ctx.copy_text(row.key.clone());
                            }
                            if ui
                                .label(format!("{}", row.value))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                ctx.copy_text(row.key.clone());
                            }
                            ui.end_row();
                        }
                    });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            }
        });
    }
}
