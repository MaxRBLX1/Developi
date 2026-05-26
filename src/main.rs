// main.rs — developi 1.0 Main Application
// The Workshop For Inventors
// No walls. No limits. Just imagination.

mod canvas;
mod engine;
mod blocks;
mod sensory;
mod project;

use eframe::egui;
use log::info;

// ─── STARTUP CHECK ───
fn verify_python_embedded() -> bool {
    use std::path::PathBuf;
    
    let exe_dir = match std::env::current_exe() {
        Ok(p) => match p.parent() {
            Some(d) => d.to_path_buf(),
            None => return false,
        },
        Err(_) => return false,
    };
    
    let python_dir = exe_dir.join("Languages").join("Python");
    
    if !python_dir.exists() {
        println!("❌ Languages\\Python folder not found at: {:?}", python_dir);
        return false;
    }
    
    let mut found_python_dll = false;
    let mut file_count = 0;
    
    // Walk ALL directories recursively
    fn walk_dir(dir: &std::path::Path, found_dll: &mut bool, file_count: &mut usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                
                if name.contains("python") && (name.ends_with(".dll") || name.ends_with(".so")) {
                    *found_dll = true;
                }
                
                if path.is_dir() {
                    walk_dir(&path, found_dll, file_count);
                } else {
                    *file_count += 1;
                }
            }
        }
    }
    
    walk_dir(&python_dir, &mut found_python_dll, &mut file_count);
    
    println!("   Found {} files, Python DLL: {}", file_count, found_python_dll);
    found_python_dll && file_count > 10
}

// ─── BUILD TEST CANVAS FOR BLOCK VALIDATION ───
struct TestBlock {
    id: u64,
    definition: blocks::BlockDefinition,
    input_values: Vec<String>,
}

fn build_test_canvas(
    target: &blocks::BlockDefinition,
    registry: &blocks::BlockRegistry,
) -> (Vec<TestBlock>, Vec<engine::ConnectionData>) {
    let mut blocks = Vec::new();
    let mut connections = Vec::new();
    let mut next_id: u64 = 100; // target gets id 100, helpers start at 1

    // Helper to add a block by name and return its id
    let mut add_block = |name: &str, inputs: Vec<String>| -> u64 {
        let def = registry.find_block(name).unwrap().clone();
        let id = if name == target.name { 100 } else { next_id };
        if name != target.name { next_id += 1; }
        blocks.push(TestBlock { id, definition: def, input_values: inputs });
        id
    };

    // Default input values for the target block
    let target_inputs: Vec<String> = target.inputs.iter()
        .map(|p| p.default_value.clone())
        .collect();

    // Determine which supporting blocks are needed
    let target_name = target.name.as_str();
    match target_name {
        "Write Memory" | "Read Memory" | "Cast Pointer" | "Free Memory" => {
            let alloc_id = add_block("Allocate Memory", vec!["64".into()]);
            let target_id = add_block(target_name, target_inputs);
            // Wire Allocate Memory address -> target address
            connections.push(engine::ConnectionData {
                from_block: alloc_id,
                from_port_index: 0, // address output
                to_block: target_id,
                to_port_index: 0,   // address input
            });
        }
        "Open File" => {
            let tmp_path = std::env::temp_dir().join("developi_test.txt");
            std::fs::write(&tmp_path, "developi test").ok();
            let path_str = tmp_path.to_string_lossy().to_string();
            add_block("Open File", vec![path_str, "r".into()]);
            std::fs::remove_file(&tmp_path).ok();
        }
        "Seek File" | "Close File" => {
            // Create a real temporary file so Open File succeeds
            let tmp_path = std::env::temp_dir().join("developi_test.txt");
            std::fs::write(&tmp_path, "developi test").ok();
            let path_str = tmp_path.to_string_lossy().to_string();

            let open_id = add_block("Open File", vec![path_str, "r".into()]);
            let target_id = add_block(target_name, target_inputs);
            connections.push(engine::ConnectionData {
                from_block: open_id,
                from_port_index: 0,
                to_block: target_id,
                to_port_index: 0,
            });

            // Cleanup after test
            std::fs::remove_file(&tmp_path).ok();
        }
        "Bind Socket" | "Send Data" | "Receive Data" | "Close Socket" | "Listen Socket" | "Accept Connection" => {
            let sock_id = add_block("Create Socket", vec!["tcp".into()]);
            let target_id = add_block(target_name, target_inputs);
            // Wire Create Socket socket -> target socket
            connections.push(engine::ConnectionData {
                from_block: sock_id,
                from_port_index: 0, // socket
                to_block: target_id,
                to_port_index: 0,   // socket
            });
        }
        "Call C Function" => {
            let lib_id = add_block("Load Library", vec!["kernel32.dll".into()]);
            let target_id = add_block(target_name, target_inputs);
            connections.push(engine::ConnectionData {
                from_block: lib_id,
                from_port_index: 0, // library
                to_block: target_id,
                to_port_index: 0,   // library
            });
        }
        _ => {
            // For all other blocks (standalone), just add the target
            add_block(target_name, target_inputs);
        }
    }

    (blocks, connections)
}

fn main() {
    // ─── STARTUP CHECK ───
    print!("🔍 Please wait while I check all the embedded things here Languages\\Python...");
    let verified = verify_python_embedded();
    
    if !verified {
        println!();
        eprintln!("❌ Error: Python embedded files not found in Languages\\Python");
        eprintln!("   Make sure the Languages\\Python folder is next to developi.exe");
        std::process::exit(1);
    }
    println!(" ✅ Verified.");
    // ─── END STARTUP CHECK ───

    // Set Python path FIRST — must happen before any Python code runs
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let python_home = exe_dir.join("Languages").join("Python");
            std::env::set_var("PYTHONHOME", &python_home);
        }
    }

    // ─── INTERNAL VALIDATION MODE ───
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--validate") {
        validate_all_blocks();
        return;
    }

    env_logger::init();
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

// ─── INTERNAL BLOCK VALIDATOR – TESTS EVERY BLOCK ───────────────
fn validate_all_blocks() {
    println!("🔍 developi 1.0 Block Validator\n");

    let registry = blocks::BlockRegistry::new();
    let all_blocks = registry.all_blocks().to_vec();

    println!("📦 Testing {} blocks...\n", all_blocks.len());

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();
    let mut port_mismatches = Vec::new();

    for block in &all_blocks {
        // Build a minimal test canvas for this block (including any needed supporting blocks)
        let (test_blocks, test_connections) = build_test_canvas(block, &registry);

        let mut engine = engine::PythonEngine::new();
        let mut block_data: Vec<engine::PlacedBlockData> = test_blocks
            .iter()
            .map(|b| engine::PlacedBlockData {
                id: b.id,
                definition: b.definition.clone(),
                input_values: b.input_values.clone(),
                output_values: vec![],
            })
            .collect();

        let result = engine.execute_dataflow(&mut block_data, &test_connections);
        let has_error = result.iter().any(|l| l.contains("✗ Error"));

        if has_error {
            failed += 1;
            let err = result.iter().find(|l| l.contains("✗ Error")).unwrap().clone();
            println!("❌ [{}] {} — {}", block.category, block.name, err);
            failures.push((block.name.clone(), block.category.clone(), err));
        } else {
            // Optional port‑count check
            let output_count = block_data.iter().find(|b| b.id == 100).map(|b| b.output_values.len()).unwrap_or(0);
            let expected = block.outputs.len();
            if output_count != expected && expected > 1 {
                port_mismatches.push(format!(
                    "⚠️  [{}] {} — {} ports expected, {} produced: {:?}",
                    block.category, block.name, expected, output_count,
                    block_data.iter().find(|b| b.id == 100).map(|b| &b.output_values).unwrap_or(&vec![])
                ));
            }
            passed += 1;
        }
    }

    println!("\n══════════════════════════════════");
    println!("📊 RESULTS: ✅ {} passed | ❌ {} failed", passed, failed);
    println!("⏭️  0 skipped (all blocks tested)");
    println!("══════════════════════════════════");

    if !port_mismatches.is_empty() {
        println!("\n── Port Mismatches (non‑fatal) ──");
        for p in &port_mismatches { println!("{}", p); }
    }

    if !failures.is_empty() {
        println!("\n── Failed Blocks ──");
        for (name, cat, err) in &failures {
            println!("  [{}] {} — {}", cat, name, err);
        }
    }

    if failed == 0 {
        println!("\n🎉 ALL BLOCKS VALIDATED!");
    }
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
                        let fixed: Vec<canvas::Connection> = conns.iter().enumerate().map(|(i, c)| canvas::Connection {
                            id: i as u64 + 1,
                            from_block: c.from_block, from_port_index: c.from_port_index,
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
                    self.is_running = true;
                    self.status_message = "Executing...".into();
                    let canvas_conns = self.canvas.get_connections();
                    let blocks = self.canvas.get_blocks_data();
                    let mut conns: Vec<engine::ConnectionData> = canvas_conns.iter().map(|c| engine::ConnectionData {
                        from_block: c.from_block, from_port_index: c.from_port_index,
                        to_block: c.to_block, to_port_index: c.to_port_index,
                    }).collect();
                    let fix_messages = self.python_engine.auto_fix_connections(&blocks, &mut conns);
                    if !fix_messages.is_empty() {
                        let fixed: Vec<canvas::Connection> = conns.iter().enumerate().map(|(i, c)| canvas::Connection {
                            id: i as u64 + 1,
                            from_block: c.from_block, from_port_index: c.from_port_index,
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
                    self.is_running = false;
                    self.status_message = "Ready".into();
                }

                if ui.button("⏹ Stop").clicked() {
                    self.status_message = "Execution stopped.".into();
                    self.console_output.push("⏹ Execution stopped by user.".into());
                    self.canvas.reset_execution_state();
                    self.is_running = false;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("🐍 Python 3.14 | developi 1.0");
                });
            });
            ui.add_space(4.0);
        });
    }

    fn open_project(&mut self, path: &std::path::PathBuf) {
        let path_str = path.display().to_string();
        match project::Project::load(path) {
            Ok(loaded) => {
                self.canvas.clear_all();
                
                for snap in &loaded.blocks {
                    if let Some(def) = self.block_registry.find_block(&snap.block_name) {
                        self.canvas.add_block_with_id(def.clone(), snap.id);
                    }
                }
                
                for snap in &loaded.blocks {
                    self.canvas.set_block_position(snap.id, snap.position_x, snap.position_y);
                }
                
                for snap in &loaded.blocks {
                    if let Some(ref code) = snap.custom_code {
                        if let Ok(vals) = serde_json::from_str::<Vec<String>>(code) {
                            self.canvas.set_block_inputs(snap.id, &vals);
                        }
                    }
                }
                
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