// canvas.rs — The Workbench
// Block rendering, dragging, input editing, connections, zoom, grid, deletion.
// Data flow execution — blocks execute in dependency order, values pass through wires.
// Port highlighting — golden glow on selected output, green pulse on compatible inputs.
// Save/Load/Auto-Fix support — get_connections, set_connections, get_blocks_data, restore_connections.
// Production code. Ships as-is for developi 1.0.

use egui::{Pos2, Rect, Stroke, Vec2, Color32, Align2, Key, PointerButton};
use log::info;
use crate::blocks::BlockDefinition;
use crate::engine::{PythonEngine, PlacedBlockData, ConnectionData};
use crate::project::ConnectionSnapshot;

pub struct CanvasState {
    blocks: Vec<PlacedBlock>,
    connections: Vec<Connection>,
    next_id: u64,
    view_offset: Vec2,
    zoom: f32,
    dragging_block: Option<u64>,
    drag_start: Pos2,
    block_original_pos: Option<Pos2>,
    grid_size: f32,
    background_color: Color32,
    grid_color: Color32,
    selected_block: Option<u64>,
    editing_block: Option<u64>,
    next_conn_id: u64,
    connecting_from: Option<(u64, usize)>,
    pending_removal: Option<u64>,
}

#[derive(Clone)]
pub struct PlacedBlock {
    pub id: u64,
    pub definition: BlockDefinition,
    pub position: Pos2,
    pub size: Vec2,
    pub is_executing: bool,
    pub input_values: Vec<String>,
    pub output_values: Vec<String>,
}

#[derive(Clone)]
pub struct Connection {
    pub id: u64,
    pub from_block: u64,
    pub from_port_index: usize,
    pub to_block: u64,
    pub to_port_index: usize,
}

impl CanvasState {
    // ─── SAVE / LOAD POSITIONS & INPUTS ──────────

    /// Return block positions and input values for save
    pub fn get_block_positions(&self) -> Vec<(u64, f32, f32, Vec<String>)> {
        self.blocks.iter().map(|b| {
            (b.id, b.position.x, b.position.y, b.input_values.clone())
        }).collect()
    }

    /// Restore a single block's position after load
    pub fn set_block_position(&mut self, block_id: u64, x: f32, y: f32) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.position = Pos2::new(x, y);
        }
    }

    /// Restore input values after load
    pub fn set_block_inputs(&mut self, block_id: u64, values: &[String]) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.input_values = values.to_vec();
        }
    }

    pub fn new() -> Self {
        CanvasState {
            blocks: Vec::new(),
            connections: Vec::new(),
            next_id: 0,
            view_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_block: None,
            drag_start: Pos2::ZERO,
            block_original_pos: None,
            grid_size: 20.0,
            background_color: Color32::from_rgb(18, 18, 22),
            grid_color: Color32::from_rgb(25, 25, 32),
            selected_block: None,
            editing_block: None,
            next_conn_id: 0,
            connecting_from: None,
            pending_removal: None,
        }
    }
 
    pub fn reset_execution_state(&mut self) {
        for block in &mut self.blocks {
            block.is_executing = false;
        }
    }

    // ─── BLOCK MANAGEMENT ─────────────────────────

    pub fn add_block(&mut self, definition: BlockDefinition) {
        let id = self.next_id;
        self.next_id += 1;
        let col = (id % 4) as f32;
        let row = (id / 4) as f32;
        let base_x = 60.0 - self.view_offset.x;
        let base_y = 60.0 - self.view_offset.y;
        let position = Pos2::new(base_x + col * 220.0, base_y + row * 100.0);
        let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
        let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
        let input_values: Vec<String> = definition.inputs.iter()
            .map(|p| p.default_value.clone())
            .collect();
        let output_values: Vec<String> = vec![String::new(); definition.outputs.len()];
        self.blocks.push(PlacedBlock {
            id, definition, position, size,
            is_executing: false, input_values, output_values,
        });
        info!("Block placed: id={} name={}", id, self.blocks.last().unwrap().definition.name);
    }


/// Add a block with a specific ID (for restoring from save)
pub fn add_block_with_id(&mut self, definition: BlockDefinition, id: u64) {
    let col = (id % 4) as f32;
    let row = (id / 4) as f32;
    let base_x = 60.0 - self.view_offset.x;
    let base_y = 60.0 - self.view_offset.y;
    let position = Pos2::new(base_x + col * 220.0, base_y + row * 100.0);
    let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
    let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
    let input_values: Vec<String> = definition.inputs.iter()
        .map(|p| p.default_value.clone())
        .collect();
    let output_values: Vec<String> = vec![String::new(); definition.outputs.len()];
    self.blocks.push(PlacedBlock {
        id, definition, position, size,
        is_executing: false, input_values, output_values,
    });
    if id >= self.next_id {
        self.next_id = id + 1;
    }
    info!("Block restored: id={}", id);
}

    pub fn remove_block(&mut self, block_id: u64) {
        self.blocks.retain(|b| b.id != block_id);
        self.connections.retain(|c| c.from_block != block_id && c.to_block != block_id);
        if self.selected_block == Some(block_id) { self.selected_block = None; }
        if self.editing_block == Some(block_id) { self.editing_block = None; }
        if self.connecting_from.map_or(false, |(id, _)| id == block_id) { self.connecting_from = None; }
    }

    pub fn block_count(&self) -> usize { self.blocks.len() }
    pub fn connection_count(&self) -> usize { self.connections.len() }

    // ─── DATA ACCESS FOR SAVE / LOAD / AUTO-FIX ──

    /// Return a copy of all connections (for save/auto-fix)
    pub fn get_connections(&self) -> Vec<Connection> {
        self.connections.clone()
    }

    /// Replace all connections (after auto-fix or project load)
    pub fn set_connections(&mut self, connections: &[Connection]) {
        self.connections = connections.to_vec();
    }

    /// Return block data in engine-compatible PlacedBlockData format
    pub fn get_blocks_data(&self) -> Vec<PlacedBlockData> {
        self.blocks.iter().map(|b| PlacedBlockData {
            id: b.id,
            definition: b.definition.clone(),
            input_values: b.input_values.clone(),
            output_values: b.output_values.clone(),
        }).collect()
    }

    /// Restore connections from a loaded project file
    pub fn restore_connections(&mut self, snapshots: &[crate::project::ConnectionSnapshot]) {
    self.connections.clear();
    for snap in snapshots {
        // Find from_block and to_block
        let from_block = self.blocks.iter().find(|b| b.id == snap.from_block_id);
        let to_block = self.blocks.iter().find(|b| b.id == snap.to_block_id);
        
        let from_port_index = if let Some(block) = from_block {
            // Try to parse from_port as a number first (index), then as a name
            snap.from_port.parse::<usize>().unwrap_or_else(|_| {
                block.definition.outputs.iter()
                    .position(|p| p.name == snap.from_port)
                    .unwrap_or(0)
            })
        } else {
            0
        };
        
        let to_port_index = if let Some(block) = to_block {
            snap.to_port.parse::<usize>().unwrap_or_else(|_| {
                block.definition.inputs.iter()
                    .position(|p| p.name == snap.to_port)
                    .unwrap_or(0)
            })
        } else {
            0
        };
        
        self.connections.push(Connection {
            id: self.next_conn_id,
            from_block: snap.from_block_id,
            from_port_index,
            to_block: snap.to_block_id,
            to_port_index,
        });
        self.next_conn_id += 1;
    }
    info!("Restored {} connections", snapshots.len());
}

    /// Clear everything for a new project
    pub fn clear_all(&mut self) {
        self.blocks.clear();
        self.connections.clear();
        self.next_id = 0;
        self.next_conn_id = 0;
        self.selected_block = None;
        self.editing_block = None;
        self.connecting_from = None;
        self.pending_removal = None;
    }

    // ─── EXECUTION ────────────────────────────────

    pub fn execute_all(&mut self, engine: &mut PythonEngine) -> Vec<String> {
        let mut block_data: Vec<PlacedBlockData> = self.blocks.iter().map(|b| PlacedBlockData {
            id: b.id,
            definition: b.definition.clone(),
            input_values: b.input_values.clone(),
            output_values: Vec::new(),
        }).collect();
        let conn_data: Vec<ConnectionData> = self.connections.iter().map(|c| ConnectionData {
            from_block: c.from_block,
            from_port_index: c.from_port_index,
            to_block: c.to_block,
            to_port_index: c.to_port_index,
        }).collect();
        let result = engine.execute_dataflow(&mut block_data, &conn_data);
        // Copy output values back to canvas blocks
        for bd in &block_data {
            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == bd.id) {
                block.output_values = bd.output_values.clone();
                block.is_executing = true;
            }
        }
        result
    }

    // ─── RENDERING ────────────────────────────────

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let canvas_rect = ui.max_rect();

        // Process pending deletion
        if let Some(id) = self.pending_removal.take() {
            self.remove_block(id);
        }

        // ── Pan (middle-click or right-click drag) ──
        let response = ui.interact(canvas_rect, ui.next_auto_id(), egui::Sense::click_and_drag());
        if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
            self.view_offset += response.drag_delta();
        }

        // ── Zoom (scroll wheel) ──
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            self.zoom *= 1.0 + scroll * 0.001;
            self.zoom = self.zoom.clamp(0.3, 3.0);
        }

        let painter = ui.painter();

        // ── Coordinate transform: world → screen ──
        let transform = |p: Pos2| -> Pos2 {
            Pos2::new(
                (p.x + self.view_offset.x) * self.zoom,
                (p.y + self.view_offset.y) * self.zoom,
            )
        };

        // ── Background ──
        painter.rect_filled(canvas_rect, 0.0, self.background_color);

        // ── Grid ──
        let grid_spacing = self.grid_size * self.zoom;
        if grid_spacing > 4.0 {
            let offset_x = (self.view_offset.x * self.zoom) % grid_spacing;
            let offset_y = (self.view_offset.y * self.zoom) % grid_spacing;
            let mut x = canvas_rect.left() + offset_x;
            while x < canvas_rect.right() {
                painter.line_segment(
                    [Pos2::new(x, canvas_rect.top()), Pos2::new(x, canvas_rect.bottom())],
                    Stroke::new(0.5, self.grid_color),
                );
                x += grid_spacing;
            }
            let mut y = canvas_rect.top() + offset_y;
            while y < canvas_rect.bottom() {
                painter.line_segment(
                    [Pos2::new(canvas_rect.left(), y), Pos2::new(canvas_rect.right(), y)],
                    Stroke::new(0.5, self.grid_color),
                );
                y += grid_spacing;
            }
        }

        // ── Draw connections (wires) ──
        for conn in &self.connections {
            let from = self.blocks.iter().find(|b| b.id == conn.from_block);
            let to = self.blocks.iter().find(|b| b.id == conn.to_block);
            if let (Some(from), Some(to)) = (from, to) {
                let fy = from.position.y + 30.0 + conn.from_port_index as f32 * 22.0;
                let ty = to.position.y + 30.0 + conn.to_port_index as f32 * 22.0;
                let fp = transform(Pos2::new(from.position.x + from.size.x, fy));
                let tp = transform(Pos2::new(to.position.x, ty));
                let mx = (fp.x + tp.x) * 0.5;
                let c1 = Pos2::new(mx, fp.y);
                let c2 = Pos2::new(mx, tp.y);
                // Gradient wire
                for i in 0..20 {
                    let t1 = i as f32 / 20.0;
                    let t2 = (i + 1) as f32 / 20.0;
                    let p1 = cubic_bezier(fp, c1, c2, tp, t1);
                    let p2 = cubic_bezier(fp, c1, c2, tp, t2);
                    let alpha = (0.4 + (i as f32 / 20.0) * 0.6) as u8;
                    painter.line_segment(
                        [p1, p2],
                        Stroke::new(2.0, Color32::from_rgba_premultiplied(80, 160, 255, alpha)),
                    );
                }
            }
        }

        // ── Input handling ──
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());
        let mouse_clicked = ui.input(|i| i.pointer.button_clicked(PointerButton::Primary));
        let mouse_right_clicked = ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary));
        let delete_pressed = ui.input(|i| i.key_pressed(Key::Delete));
        let mouse_down = ui.input(|i| i.pointer.button_down(PointerButton::Primary));
        let mouse_released = ui.input(|i| i.pointer.button_released(PointerButton::Primary));

        // ── Delete key removes selected block ──
        if delete_pressed {
            self.pending_removal = self.selected_block;
        }

        // ── Right-click: select block + open properties ──
        if mouse_right_clicked {
            if let Some(mp) = mouse_pos {
                let wm = inverse_transform(mp, self.view_offset, self.zoom);
                let clicked = self.blocks.iter().rev()
                    .find(|b| Rect::from_min_size(b.position, b.size).contains(wm))
                    .map(|b| b.id);
                if let Some(id) = clicked {
                    self.selected_block = Some(id);
                    self.editing_block = Some(id);
                } else {
                    self.selected_block = None;
                    self.editing_block = None;
                }
            }
        }

        // ── Left-click: connect ports or select block ──
        if mouse_clicked {
            if let Some(mp) = mouse_pos {
                let wm = inverse_transform(mp, self.view_offset, self.zoom);
                let mut clicked_port: Option<(u64, usize, bool)> = None;
                let mut clicked_block: Option<u64> = None;

                for block in self.blocks.iter().rev() {
                    let r = Rect::from_min_size(block.position, block.size);
                    if r.contains(wm) {
                        clicked_block = Some(block.id);
                        let ry = wm.y - block.position.y;
                        // Check output ports (right side)
                        for (i, _) in block.definition.outputs.iter().enumerate() {
                            let py = 30.0 + i as f32 * 22.0;
                            if (ry - py).abs() < 14.0 && wm.x > block.position.x + block.size.x - 30.0 {
                                clicked_port = Some((block.id, i, true));
                                break;
                            }
                        }
                        // Check input ports (left side)
                        if clicked_port.is_none() {
                            for (i, _) in block.definition.inputs.iter().enumerate() {
                                let py = 30.0 + i as f32 * 22.0;
                                if (ry - py).abs() < 14.0 && wm.x < block.position.x + 30.0 {
                                    clicked_port = Some((block.id, i, false));
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }

                if let Some((block_id, port_idx, is_output)) = clicked_port {
                    if is_output {
                        // Start connecting from this output
                        self.connecting_from = Some((block_id, port_idx));
                    } else if let Some((from_id, from_idx)) = self.connecting_from.take() {
                        // Complete the connection
                        let conn = Connection {
                            id: self.next_conn_id,
                            from_block: from_id,
                            from_port_index: from_idx,
                            to_block: block_id,
                            to_port_index: port_idx,
                        };
                        self.next_conn_id += 1;
                        self.connections.push(conn);
                        info!("Connected: {}:{} -> {}:{}", from_id, from_idx, block_id, port_idx);
                    }
                } else if let Some(id) = clicked_block {
                    self.selected_block = Some(id);
                    self.editing_block = None;
                    self.connecting_from = None;
                } else {
                    self.selected_block = None;
                    self.editing_block = None;
                    self.connecting_from = None;
                }
            }
        }

        // ── Drag blocks ──
        if mouse_down && self.dragging_block.is_none() {
            if let Some(mp) = mouse_pos {
                let wm = inverse_transform(mp, self.view_offset, self.zoom);
                if let Some(block) = self.blocks.iter().rev()
                    .find(|b| Rect::from_min_size(b.position, b.size).contains(wm))
                {
                    self.dragging_block = Some(block.id);
                    self.drag_start = mp;
                    self.block_original_pos = Some(block.position);
                }
            }
        }
        if let Some(block_id) = self.dragging_block {
            if let Some(mp) = mouse_pos {
                let delta = (mp - self.drag_start) / self.zoom;
                if let Some(orig) = self.block_original_pos {
                    let snapped = Pos2::new(
                        ((orig.x + delta.x) / self.grid_size).round() * self.grid_size,
                        ((orig.y + delta.y) / self.grid_size).round() * self.grid_size,
                    );
                    if let Some(b) = self.blocks.iter_mut().find(|b| b.id == block_id) {
                        b.position = snapped;
                    }
                }
            }
        }
        if mouse_released {
            self.dragging_block = None;
            self.block_original_pos = None;
        }

        // ─── RENDER BLOCKS ──────────────────────────

        for block in &self.blocks {
            let r = Rect::from_min_size(transform(block.position), block.size * self.zoom);
            if !canvas_rect.intersects(r) { continue; }
            let sel = self.selected_block == Some(block.id);

            // Shadow
            painter.rect_filled(
                r.translate(Vec2::new(3.0, 3.0)),
                5.0,
                Color32::from_black_alpha(if sel { 100 } else { 60 }),
            );
            // Body
            let body = if block.is_executing {
                Color32::from_rgb(60, 100, 60)
            } else if sel {
                Color32::from_rgb(55, 55, 75)
            } else {
                Color32::from_rgb(40, 40, 52)
            };
            painter.rect_filled(r, 5.0, body);
            // Border
            let border = if sel {
                Color32::from_rgb(100, 160, 255)
            } else if block.is_executing {
                Color32::from_rgb(100, 200, 100)
            } else {
                Color32::from_rgb(60, 60, 75)
            };
            painter.rect_stroke(r, 5.0, Stroke::new(1.5, border));
            // Header
            let hr = Rect::from_min_max(r.min, Pos2::new(r.max.x, r.min.y + 24.0 * self.zoom));
            painter.rect_filled(hr, 5.0, Color32::from_rgba_premultiplied(30, 30, 40, 200));
            painter.text(
                Pos2::new(r.left() + 8.0, r.top() + 6.0 * self.zoom),
                Align2::LEFT_TOP,
                &block.definition.icon,
                egui::FontId::proportional(13.0 * self.zoom),
                Color32::WHITE,
            );
            painter.text(
                Pos2::new(r.left() + 30.0, r.top() + 5.0 * self.zoom),
                Align2::LEFT_TOP,
                &block.definition.name,
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::from_rgb(210, 210, 225),
            );

            // ── Input ports (gray circles, left side) ──
            for (i, port) in block.definition.inputs.iter().enumerate() {
                let py = r.top() + (30.0 + i as f32 * 22.0) * self.zoom;
                let dp = Pos2::new(r.left(), py);
                let is_connected = self.connections.iter()
                    .any(|c| c.to_block == block.id && c.to_port_index == i);
                let is_targeting = self.connecting_from.is_some()
                    && self.connecting_from.map_or(false, |(from_id, _)| from_id != block.id);

                let dot_color = if is_connected {
                    Color32::from_rgb(100, 220, 255)
                } else if is_targeting {
                    Color32::from_rgb(120, 255, 120)
                } else {
                    Color32::from_rgb(160, 160, 180)
                };
                let dot_radius = if is_targeting { 6.0 } else { 5.0 };
                let border_w = if is_targeting { 2.0 } else { 1.0 };

                painter.circle_filled(dp, dot_radius * self.zoom, dot_color);
                painter.circle_stroke(dp, dot_radius * self.zoom, Stroke::new(border_w, Color32::from_rgb(80, 80, 100)));

                let label = if is_connected {
                    "⟵ connected".to_string()
                } else if is_targeting {
                    "click to connect".to_string()
                } else {
                    let val = block.input_values.get(i).map(|s| s.as_str()).unwrap_or("");
                    if val.is_empty() || val == port.default_value {
                        format!("{} ({})", port.name, port.port_type)
                    } else {
                        format!("{} = {}", port.name, val)
                    }
                };
                painter.text(
                    Pos2::new(r.left() + 14.0, py - 4.0),
                    Align2::LEFT_CENTER,
                    &label,
                    egui::FontId::proportional(9.0 * self.zoom),
                    Color32::from_rgb(160, 160, 180),
                );
            }

            // ── Output ports (blue circles, right side) ──
            for (i, port) in block.definition.outputs.iter().enumerate() {
                let py = r.top() + (30.0 + i as f32 * 22.0) * self.zoom;
                let dp = Pos2::new(r.right(), py);
                let is_connected = self.connections.iter()
                    .any(|c| c.from_block == block.id && c.from_port_index == i);
                let is_selected = self.connecting_from
                    .map_or(false, |(id, idx)| id == block.id && idx == i);

                let dot_color = if is_connected {
                    Color32::from_rgb(100, 220, 100)
                } else if is_selected {
                    Color32::from_rgb(255, 200, 50)
                } else {
                    Color32::from_rgb(100, 180, 255)
                };
                let dot_radius = if is_selected { 7.0 } else { 5.0 };
                let border_w = if is_selected { 2.0 } else { 1.0 };

                painter.circle_filled(dp, dot_radius * self.zoom, dot_color);
                painter.circle_stroke(dp, dot_radius * self.zoom, Stroke::new(border_w, Color32::from_rgb(50, 120, 200)));

                let out_val = block.output_values.get(i).map(|s| s.as_str()).unwrap_or("");
                let label = if out_val.is_empty() {
                    port.name.clone()
                } else {
                    format!("{} = {}", port.name, out_val)
                };
                painter.text(
                    Pos2::new(r.right() - 14.0, py - 4.0),
                    Align2::RIGHT_CENTER,
                    &label,
                    egui::FontId::proportional(9.0 * self.zoom),
                    Color32::from_rgb(160, 180, 210),
                );
            }
        }

        // ─── PROPERTIES PANEL (right-click a block) ──
        if let Some(edit_id) = self.editing_block {
            let mut pending_delete = false;
            let block_name;
            let mut edited_values: Vec<String>;
            let input_defs: Vec<(String, String)>;

            if let Some(block) = self.blocks.iter().find(|b| b.id == edit_id) {
                block_name = block.definition.name.clone();
                input_defs = block.definition.inputs.iter()
                    .map(|p| (p.name.clone(), p.port_type.clone()))
                    .collect();
                edited_values = block.input_values.iter()
                    .map(|v| if is_connected_value(v) { String::new() } else { v.clone() })
                    .collect();
            } else {
                return;
            }

            egui::SidePanel::right("properties_panel")
                .default_width(250.0)
                .show(ui.ctx(), |ui| {
                    ui.heading(&block_name);
                    ui.separator();
                    for (i, (name, ptype)) in input_defs.iter().enumerate() {
                        let is_conn = i < edited_values.len()
                            && is_connected_value(
                                &self.blocks.iter()
                                    .find(|b| b.id == edit_id)
                                    .and_then(|b| b.input_values.get(i))
                                    .cloned()
                                    .unwrap_or_default()
                            );
                        if is_conn {
                            ui.label(format!("{} ({}) ⟵ connected", name, ptype));
                        } else {
                            ui.label(format!("{} ({})", name, ptype));
                            if i < edited_values.len() {
                                ui.text_edit_singleline(&mut edited_values[i]);
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("🗑️ Delete Block").clicked() {
                        pending_delete = true;
                    }
                });

            // Apply edits
            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == edit_id) {
                for (i, val) in edited_values.iter().enumerate() {
                    if !is_connected_value(
                        &block.input_values.get(i).cloned().unwrap_or_default()
                    ) {
                        if i < block.input_values.len() {
                            block.input_values[i] = val.clone();
                        }
                    }
                }
            }
            if pending_delete {
                self.pending_removal = Some(edit_id);
            }
        }
    }
}

// ─── HELPER FUNCTIONS ───────────────────────────────

fn is_connected_value(val: &str) -> bool {
    val.starts_with("{{") && val.ends_with("}}")
}

fn inverse_transform(screen: Pos2, offset: Vec2, zoom: f32) -> Pos2 {
    Pos2::new(
        screen.x / zoom - offset.x,
        screen.y / zoom - offset.y,
    )
}

fn cubic_bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    Pos2::new(
        u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x,
        u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y,
    )
}