mod canvas;
mod engine;
mod blocks;
mod sensory;
mod project;

use eframe::egui;
use log::info;

fn main() {
    env_logger::init();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let python_home = exe_dir.join("languages").join("python");
            std::env::set_var("PYTHONHOME", &python_home);
            info!("Python home set to: {:?}", python_home);
        }
    }

    info!("developi 1.0 starting...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("developi — The Workshop"),
        ..Default::default()
    };

    eframe::run_native(
        "developi",
        options,
        Box::new(|_cc| {
            info!("Workshop open. The functions are ready.");
            Box::new(DevelopiApp::new())
        }),
    )
    .expect("developi failed to start.");
}

pub struct DevelopiApp {
    canvas: canvas::CanvasState,
    block_registry: blocks::BlockRegistry,
    python_engine: engine::PythonEngine,
    sensory: sensory::WorkshopSensory,
    console_output: Vec<String>,
    status_message: String,
    is_running: bool,
    project_path: Option<String>,
}

impl DevelopiApp {
    pub fn new() -> Self {
        let sensory = sensory::WorkshopSensory::new();
        sensory.play_startup();

        DevelopiApp {
            canvas: canvas::CanvasState::new(),
            block_registry: blocks::BlockRegistry::new(),
            python_engine: engine::PythonEngine::new(),
            sensory,
            console_output: vec![
                "developi 1.0 ready.".into(),
                "Python 3.14 engine loaded.".into(),
                "💡 Click 🔵 then ⚪ to connect blocks.".into(),
                "💡 Right-click any block to edit its properties.".into(),
            ],
            status_message: "Ready".into(),
            is_running: false,
            project_path: None,
        }
    }
}

impl eframe::App for DevelopiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.render_toolbar(ctx);
        self.render_block_palette(ctx);
        self.render_canvas(ctx);
        self.render_console(ctx);
        self.render_status_bar(ctx);
    }
}

impl DevelopiApp {
    fn render_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🔧 developi");
                ui.separator();

                if ui.button("🆕 New").clicked() {
                    self.canvas.clear_all();
                    self.project_path = None;
                    self.console_output.clear();
                    self.console_output.push("🆕 New project created.".into());
                    self.status_message = "New project".into();
                }

                if ui.button("📁 Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("developi Project", &["dev"])
                        .pick_file()
                    {
                        self.open_project(&path);
                    }
                }

                if ui.button("💾 Save").clicked() {
                    self.save_project();
                }

                if ui.button("📄 Save As").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("developi Project", &["dev"])
                        .save_file()
                    {
                        self.project_path = Some(path.display().to_string());
                        self.save_project();
                    }
                }

                ui.separator();

                if ui.button("🔧 Auto-Fix").clicked() {
                    self.status_message = "Analyzing connections...".into();
                    let canvas_conns = self.canvas.get_connections();
                    let blocks = self.canvas.get_blocks_data();
                    let mut conns: Vec<engine::ConnectionData> = canvas_conns.iter().map(|c| engine::ConnectionData {
                        from_block: c.from_block, from_port_index: c.from_port_index,
                        to_block: c.to_block, to_port_index: c.to_port_index,
                    }).collect();
                    let fix_messages = self.python_engine.auto_fix_connections(&blocks, &mut conns);
                    if fix_messages.is_empty() {
                        self.console_output.push("✅ No wiring issues found.".into());
                        self.status_message = "All connections look good!".into();
                    } else {
                        let fixed: Vec<canvas::Connection> = conns.iter().map(|c| canvas::Connection {
                            id: 0, from_block: c.from_block, from_port_index: c.from_port_index,
                            to_block: c.to_block, to_port_index: c.to_port_index,
                        }).collect();
                        self.canvas.set_connections(&fixed);
                        self.console_output.push("── Auto-Fix Results ──".into());
                        for msg in &fix_messages { self.console_output.push(msg.clone()); }
                        self.status_message = format!("Fixed {} connection(s)", fix_messages.len());
                    }
                }

                ui.separator();

                if ui.button("▶ Run All").clicked() {
                    self.status_message = "Executing...".into();
                    let canvas_conns = self.canvas.get_connections();
                    let blocks = self.canvas.get_blocks_data();
                    let mut conns: Vec<engine::ConnectionData> = canvas_conns.iter().map(|c| engine::ConnectionData {
                        from_block: c.from_block, from_port_index: c.from_port_index,
                        to_block: c.to_block, to_port_index: c.to_port_index,
                    }).collect();
                    let fix_messages = self.python_engine.auto_fix_connections(&blocks, &mut conns);
                    if !fix_messages.is_empty() {
                        let fixed: Vec<canvas::Connection> = conns.iter().map(|c| canvas::Connection {
                            id: 0, from_block: c.from_block, from_port_index: c.from_port_index,
                            to_block: c.to_block, to_port_index: c.to_port_index,
                        }).collect();
                        self.canvas.set_connections(&fixed);
                        self.console_output.push("── Auto-Fix ──".into());
                        for msg in &fix_messages { self.console_output.push(msg.clone()); }
                    }
                    let output = self.canvas.execute_all(&mut self.python_engine);
                    self.console_output.push("── Execution ──".into());
                    self.console_output.extend(output);
                    self.console_output.push("── Complete ──".into());
                    self.canvas.reset_execution_state();
                    self.status_message = "Ready".into();
                }

                if ui.button("⏹ Stop").clicked() {
                    self.status_message = "Execution stopped.".into();
                    self.console_output.push("⏹ Execution stopped by user.".into());
                    self.canvas.reset_execution_state();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("🐍 Python 3.14 | developi 1.0");
                });
            });
            ui.add_space(4.0);
        });
    }

    // ─── OPEN PROJECT ──────────────────────────

    fn open_project(&mut self, path: &std::path::PathBuf) {
        let path_str = path.display().to_string();
        match project::Project::load(path) {
            Ok(loaded) => {
                self.canvas.clear_all();
                
                // Step 1: Restore blocks with original IDs
                for snap in &loaded.blocks {
                    if let Some(def) = self.block_registry.find_block(&snap.block_name) {
                        self.canvas.add_block_with_id(def.clone(), snap.id);
                    }
                }
                
                // Step 2: Restore positions FIRST
                for snap in &loaded.blocks {
                    self.canvas.set_block_position(snap.id, snap.position_x, snap.position_y);
                }
                
                // Step 3: Restore input values SECOND
                for snap in &loaded.blocks {
                    if let Some(ref code) = snap.custom_code {
                        if let Ok(vals) = serde_json::from_str::<Vec<String>>(code) {
                            self.canvas.set_block_inputs(snap.id, &vals);
                        }
                    }
                }
                
                // Step 4: Restore connections LAST
                if !loaded.connections.is_empty() {
                    self.canvas.restore_connections(&loaded.connections);
                }
                
                self.project_path = Some(path_str.clone());
                self.status_message = format!("Loaded: {}", path_str);
                self.console_output.push(format!("📂 Opened: {} ({} blocks, {} connections)", 
                    path_str, loaded.blocks.len(), loaded.connections.len()));
            }
            Err(e) => {
                self.status_message = format!("Open failed: {}", e);
                self.console_output.push(format!("❌ Open error: {}", e));
            }
        }
    }

    // ─── SAVE PROJECT ──────────────────────────

    fn save_project(&mut self) {
        let path = self.project_path.clone().unwrap_or_else(|| "project.dev".to_string());
        let blocks_data = self.canvas.get_blocks_data();
        let connections = self.canvas.get_connections();
        
        let block_snapshots: Vec<project::PlacedBlockSnapshot> = self.canvas.get_block_positions()
            .iter().map(|(id, x, y, input_vals)| {
                let block_name = blocks_data.iter().find(|b| b.id == *id)
                    .map(|b| b.definition.name.clone()).unwrap_or_default();
                let category = blocks_data.iter().find(|b| b.id == *id)
                    .map(|b| b.definition.category.clone()).unwrap_or_default();
                project::PlacedBlockSnapshot {
                    id: *id, block_name, category,
                    position_x: *x, position_y: *y,
                    size_x: 200.0, size_y: 100.0,
                    custom_code: Some(serde_json::to_string(input_vals).unwrap_or_default()),
                }
            }).collect();
            
        let conn_snapshots: Vec<project::ConnectionSnapshot> = connections.iter().map(|c| {
            project::ConnectionSnapshot {
                id: c.id,
                from_block_id: c.from_block,
                from_port: c.from_port_index.to_string(),
                to_block_id: c.to_block,
                to_port: c.to_port_index.to_string(),
            }
        }).collect();
        
        let project = project::Project {
            metadata: project::ProjectMetadata {
                name: "developi_project".into(),
                version: "1.0".into(),
                developi_version: "1.0".into(),
                created_at: String::new(),
                modified_at: String::new(),
                block_count: self.canvas.block_count(),
                connection_count: self.canvas.connection_count(),
                description: String::new(),
            },
            canvas: project::CanvasSnapshot {
                zoom: 1.0, view_offset_x: 0.0, view_offset_y: 0.0,
                grid_size: 20.0, grid_visible: true,
            },
            blocks: block_snapshots,
            connections: conn_snapshots,
            variables: vec![],
            sensory: project::SensorySnapshot::default(),
        };
        
        let path_buf = std::path::PathBuf::from(&path);
        match project.save(&path_buf) {
            Ok(()) => {
                self.project_path = Some(path);
                self.status_message = "Project saved.".into();
                self.console_output.push(format!("💾 Saved: {}", path_buf.display()));
            }
            Err(e) => {
                self.status_message = format!("Save failed: {}", e);
                self.console_output.push(format!("❌ Save error: {}", e));
            }
        }
    }

    fn render_block_palette(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("block_palette").default_width(220.0).resizable(true).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("📦 Block Palette");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let categories = self.block_registry.categories().to_vec();
                for category in &categories {
                    egui::collapsing_header::CollapsingHeader::new(format!("{}  {}", category.icon, category.name))
                        .default_open(true)
                        .show(ui, |ui| {
                            for block in self.block_registry.blocks_in_category(category) {
                                let btn = egui::Button::new(format!("  {}  {}", block.icon, block.name))
                                    .min_size(egui::vec2(ui.available_width(), 28.0));
                                if ui.add(btn).clicked() {
                                    self.canvas.add_block(block.clone());
                                    self.sensory.play_block_place();
                                    self.status_message = format!("Placed: {}", block.name);
                                    self.console_output.push(format!("+ Block placed: {}", block.name));
                                }
                            }
                        });
                }
            });
        });
    }

    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| { self.canvas.render(ui); });
    }

    fn render_console(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("console").default_height(150.0).resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| { ui.heading("📋 Console"); if ui.button("Clear").clicked() { self.console_output.clear(); } });
            ui.separator();
            egui::ScrollArea::vertical().max_height(ui.available_height()).stick_to_bottom(true).show(ui, |ui| {
                for line in &self.console_output { ui.label(line); }
            });
        });
    }

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").min_height(24.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("📦 Blocks: {}", self.canvas.block_count()));
                ui.separator();
                ui.label(format!("🔗 Connections: {}", self.canvas.connection_count()));
                ui.separator();
                ui.label("💡 Click 🔵 then ⚪ to connect | Right-click block = properties");
                ui.separator();
                ui.label(format!("💾 RAM: {}MB", self.python_engine.ram_usage()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_running { ui.label("⏳ Running..."); } else { ui.label(&self.status_message.clone()); }
                });
            });
        });
    }
}