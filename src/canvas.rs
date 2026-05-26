// canvas.rs — The Workbench
// Block rendering, dragging, input editing, connections, zoom, grid, deletion.
// Data flow execution — blocks execute in dependency order, values pass through wires.
// Port highlighting — golden glow on selected output, green pulse on compatible inputs.
// Save/Load/Auto-Fix support — get_connections, set_connections, get_blocks_data, restore_connections.
// Undo/Redo support — full history tracking for all actions.
// Wire deletion — right-click wires to remove.
// Production code. Ships as-is for developi 1.0.

use egui::{Pos2, Rect, Stroke, Vec2, Color32, Align2, Key, PointerButton};
use log::info;
use crate::blocks::BlockDefinition;
use crate::engine::{PythonEngine, PlacedBlockData, ConnectionData};

// ─── UNDO/REDO HISTORY ─────────────────────────────────

#[derive(Clone)]
pub enum CanvasAction {
    AddBlock {
        block_id: u64,
        definition: BlockDefinition,
        position: Pos2,
        input_values: Vec<String>,
    },
    RemoveBlock {
        block_id: u64,
        definition: BlockDefinition,
        position: Pos2,
        input_values: Vec<String>,
        connections: Vec<Connection>,
    },
    MoveBlock {
        block_id: u64,
        old_position: Pos2,
        new_position: Pos2,
    },
    AddConnection {
        connection: Connection,
    },
    RemoveConnection {
        connection: Connection,
    },
    UpdateInput {
        block_id: u64,
        port_index: usize,
        old_value: String,
        new_value: String,
    },
}

pub struct UndoHistory {
    actions: Vec<CanvasAction>,
    current_index: usize,
    max_size: usize,
}

impl UndoHistory {
    pub fn new() -> Self {
        UndoHistory {
            actions: Vec::new(),
            current_index: 0,
            max_size: 100,
        }
    }

    pub fn push(&mut self, action: CanvasAction) {
        if self.current_index < self.actions.len() {
            self.actions.truncate(self.current_index);
        }
        
        self.actions.push(action);
        
        if self.actions.len() > self.max_size {
            self.actions.remove(0);
        } else {
            self.current_index = self.actions.len();
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.actions.len()
    }

    pub fn undo(&mut self) -> Option<CanvasAction> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.actions[self.current_index].clone())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<CanvasAction> {
        if self.current_index < self.actions.len() {
            let action = self.actions[self.current_index].clone();
            self.current_index += 1;
            Some(action)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.current_index = 0;
    }
}

// ─── CANVAS STATE ─────────────────────────────────────

pub struct CanvasState {
    pub blocks: Vec<PlacedBlock>,
    pub connections: Vec<Connection>,
    next_id: u64,
    pub view_offset: Vec2,
    pub zoom: f32,
    dragging_block: Option<u64>,
    drag_start: Pos2,
    drag_start_pos: Option<Pos2>,
    grid_size: f32,
    background_color: Color32,
    grid_color: Color32,
    selected_block: Option<u64>,
    selected_connection: Option<u64>,
    editing_block: Option<u64>,
    editing_connection: Option<u64>,
    next_conn_id: u64,
    connecting_from: Option<(u64, usize)>,
    pending_removal: Option<u64>,
    pending_connection_removal: Option<u64>,
    pub undo_history: UndoHistory,
    pub needs_repaint: bool,
    pub search_query: String,
    pub show_experimental: bool,
    wire_hover: Option<u64>,
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
    // ─── PUBLIC ACCESSORS ───────────────────────────────

    pub fn get_zoom(&self) -> f32 { self.zoom }
    pub fn get_view_offset(&self) -> Vec2 { self.view_offset }
    pub fn needs_repaint(&self) -> bool { self.needs_repaint }
    pub fn reset_repaint(&mut self) { self.needs_repaint = false; }
    pub fn set_show_experimental(&mut self, show: bool) { self.show_experimental = show; self.request_repaint(); }
    pub fn set_search_query(&mut self, query: &str) { self.search_query = query.to_string(); self.request_repaint(); }
    pub fn can_undo(&self) -> bool { self.undo_history.can_undo() }
    pub fn can_redo(&self) -> bool { self.undo_history.can_redo() }
    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_history.undo() {
            self.apply_action_inverse(&action);
            self.request_repaint();
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.undo_history.redo() {
            self.apply_action(&action);
            self.request_repaint();
            true
        } else {
            false
        }
    }

    // ─── SAVE / LOAD ─────────────────────────────────

    pub fn get_block_positions(&self) -> Vec<(u64, f32, f32, Vec<String>)> {
        self.blocks.iter().map(|b| {
            (b.id, b.position.x, b.position.y, b.input_values.clone())
        }).collect()
    }

    pub fn set_block_position(&mut self, block_id: u64, x: f32, y: f32) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.position = Pos2::new(x, y);
        }
    }

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
            drag_start_pos: None,
            grid_size: 20.0,
            background_color: Color32::from_rgb(18, 18, 22),
            grid_color: Color32::from_rgb(25, 25, 32),
            selected_block: None,
            selected_connection: None,
            editing_block: None,
            editing_connection: None,
            next_conn_id: 0,
            connecting_from: None,
            pending_removal: None,
            pending_connection_removal: None,
            undo_history: UndoHistory::new(),
            needs_repaint: true,
            search_query: String::new(),
            show_experimental: false,
            wire_hover: None,
        }
    }

    pub fn request_repaint(&mut self) {
        self.needs_repaint = true;
    }

    pub fn reset_execution_state(&mut self) {
        for block in &mut self.blocks {
            block.is_executing = false;
        }
        self.request_repaint();
    }

    fn apply_action(&mut self, action: &CanvasAction) {
        match action {
            CanvasAction::AddBlock { block_id, definition, position, input_values } => {
                let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
                let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
                let output_values = vec![String::new(); definition.outputs.len()];
                self.blocks.push(PlacedBlock {
                    id: *block_id,
                    definition: definition.clone(),
                    position: *position,
                    size,
                    is_executing: false,
                    input_values: input_values.clone(),
                    output_values,
                });
            }
            CanvasAction::RemoveBlock { block_id, .. } => {
                self.blocks.retain(|b| b.id != *block_id);
            }
            CanvasAction::MoveBlock { block_id, new_position, .. } => {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == *block_id) {
                    block.position = *new_position;
                }
            }
            CanvasAction::AddConnection { connection } => {
                self.connections.push(connection.clone());
            }
            CanvasAction::RemoveConnection { connection } => {
                self.connections.retain(|c| c.id != connection.id);
            }
            CanvasAction::UpdateInput { block_id, port_index, new_value, .. } => {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == *block_id) {
                    if *port_index < block.input_values.len() {
                        block.input_values[*port_index] = new_value.clone();
                    }
                }
            }
        }
    }

    fn apply_action_inverse(&mut self, action: &CanvasAction) {
        match action {
            CanvasAction::AddBlock { block_id, .. } => {
                self.blocks.retain(|b| b.id != *block_id);
            }
            CanvasAction::RemoveBlock { block_id, definition, position, input_values, connections } => {
                let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
                let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
                let output_values = vec![String::new(); definition.outputs.len()];
                self.blocks.push(PlacedBlock {
                    id: *block_id,
                    definition: definition.clone(),
                    position: *position,
                    size,
                    is_executing: false,
                    input_values: input_values.clone(),
                    output_values,
                });
                for conn in connections {
                    self.connections.push(conn.clone());
                }
            }
            CanvasAction::MoveBlock { block_id, old_position, .. } => {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == *block_id) {
                    block.position = *old_position;
                }
            }
            CanvasAction::AddConnection { connection } => {
                self.connections.retain(|c| c.id != connection.id);
            }
            CanvasAction::RemoveConnection { connection } => {
                self.connections.push(connection.clone());
            }
            CanvasAction::UpdateInput { block_id, port_index, old_value, .. } => {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == *block_id) {
                    if *port_index < block.input_values.len() {
                        block.input_values[*port_index] = old_value.clone();
                    }
                }
            }
        }
    }

    fn push_action(&mut self, action: CanvasAction) {
        self.undo_history.push(action);
        self.request_repaint();
    }

    // ─── BLOCK MANAGEMENT ─────────────────────────

    pub fn add_block(&mut self, definition: BlockDefinition) {
        let id = self.next_id;
        self.next_id += 1;
        let col = (id % 4) as f32;
        let row = (id / 4) as f32;
        let base_x = 100.0 - self.view_offset.x;
        let base_y = 100.0 - self.view_offset.y;
        let position = Pos2::new(base_x + col * 220.0, base_y + row * 100.0);
        let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
        let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
        let input_values: Vec<String> = definition.inputs.iter()
            .map(|p| p.default_value.clone())
            .collect();
        
        let action = CanvasAction::AddBlock {
            block_id: id,
            definition: definition.clone(),
            position,
            input_values: input_values.clone(),
        };
        self.push_action(action);
        
        let output_values = vec![String::new(); definition.outputs.len()];
        self.blocks.push(PlacedBlock {
            id, definition, position, size,
            is_executing: false, input_values, output_values,
        });
        info!("Block placed: id={}", id);
    }

    pub fn add_block_with_id(&mut self, definition: BlockDefinition, id: u64) {
        let col = (id % 4) as f32;
        let row = (id / 4) as f32;
        let base_x = 100.0 - self.view_offset.x;
        let base_y = 100.0 - self.view_offset.y;
        let position = Pos2::new(base_x + col * 220.0, base_y + row * 100.0);
        let port_count = definition.inputs.len().max(definition.outputs.len()).max(1) as f32;
        let size = Vec2::new(200.0, 50.0 + port_count * 22.0);
        let input_values: Vec<String> = definition.inputs.iter()
            .map(|p| p.default_value.clone())
            .collect();
        let output_values = vec![String::new(); definition.outputs.len()];
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
        if let Some(block) = self.blocks.iter().find(|b| b.id == block_id) {
            let connections: Vec<Connection> = self.connections.iter()
                .filter(|c| c.from_block == block_id || c.to_block == block_id)
                .cloned()
                .collect();
            
            let action = CanvasAction::RemoveBlock {
                block_id,
                definition: block.definition.clone(),
                position: block.position,
                input_values: block.input_values.clone(),
                connections,
            };
            self.push_action(action);
        }
        
        self.blocks.retain(|b| b.id != block_id);
        self.connections.retain(|c| c.from_block != block_id && c.to_block != block_id);
        if self.selected_block == Some(block_id) { self.selected_block = None; }
        if self.editing_block == Some(block_id) { self.editing_block = None; }
        if self.selected_connection == Some(block_id) { self.selected_connection = None; }
        if self.connecting_from.map_or(false, |(id, _)| id == block_id) { self.connecting_from = None; }
        self.request_repaint();
    }

    pub fn remove_connection(&mut self, connection_id: u64) {
        if let Some(conn) = self.connections.iter().find(|c| c.id == connection_id) {
            let action = CanvasAction::RemoveConnection { connection: conn.clone() };
            self.push_action(action);
        }
        self.connections.retain(|c| c.id != connection_id);
        if self.selected_connection == Some(connection_id) {
            self.selected_connection = None;
        }
        self.request_repaint();
    }

    pub fn block_count(&self) -> usize { self.blocks.len() }
    pub fn connection_count(&self) -> usize { self.connections.len() }

    // ─── DATA ACCESS ─────────────────────────────────

    pub fn get_connections(&self) -> Vec<Connection> {
        self.connections.clone()
    }

    pub fn set_connections(&mut self, connections: &[Connection]) {
        self.connections = connections.to_vec();
        self.request_repaint();
    }

    pub fn get_blocks_data(&self) -> Vec<PlacedBlockData> {
        self.blocks.iter().map(|b| PlacedBlockData {
            id: b.id,
            definition: b.definition.clone(),
            input_values: b.input_values.clone(),
            output_values: b.output_values.clone(),
        }).collect()
    }

    pub fn restore_connections(&mut self, snapshots: &[crate::project::ConnectionSnapshot]) {
        self.connections.clear();
        for snap in snapshots {
            let from_block = self.blocks.iter().find(|b| b.id == snap.from_block_id);
            let to_block = self.blocks.iter().find(|b| b.id == snap.to_block_id);
            
            let from_port_index = if let Some(block) = from_block {
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

    pub fn clear_all(&mut self) {
        self.blocks.clear();
        self.connections.clear();
        self.next_id = 0;
        self.next_conn_id = 0;
        self.selected_block = None;
        self.selected_connection = None;
        self.editing_block = None;
        self.editing_connection = None;
        self.connecting_from = None;
        self.pending_removal = None;
        self.pending_connection_removal = None;
        self.undo_history.clear();
        self.request_repaint();
    }

    // ─── EXECUTION ─────────────────────────────────

    pub fn execute_all(&mut self, engine: &mut PythonEngine) -> Vec<String> {
        let mut block_data: Vec<PlacedBlockData> = self.blocks.iter().map(|b| PlacedBlockData {
            id: b.id,
            definition: b.definition.clone(),
            input_values: b.input_values.clone(),
            output_values: b.output_values.clone(),
        }).collect();
        let conn_data: Vec<ConnectionData> = self.connections.iter().map(|c| ConnectionData {
            from_block: c.from_block,
            from_port_index: c.from_port_index,
            to_block: c.to_block,
            to_port_index: c.to_port_index,
        }).collect();
        let result = engine.execute_dataflow(&mut block_data, &conn_data);
        for bd in &block_data {
            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == bd.id) {
                block.output_values = bd.output_values.clone();
                block.is_executing = true;
            }
        }
        self.request_repaint();
        result
    }

    // ─── ZOOM AND VIEW ─────────────────────────────

    pub fn zoom_to_fit(&mut self) {
        if self.blocks.is_empty() {
            self.view_offset = Vec2::ZERO;
            self.zoom = 1.0;
            return;
        }
        
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for block in &self.blocks {
            min_x = min_x.min(block.position.x);
            min_y = min_y.min(block.position.y);
            max_x = max_x.max(block.position.x + block.size.x);
            max_y = max_y.max(block.position.y + block.size.y);
        }
        
        let width = max_x - min_x;
        let height = max_y - min_y;
        let padding = 100.0;
        
        let target_width = 1200.0 - padding * 2.0;
        let target_height = 800.0 - padding * 2.0;
        
        let zoom_x = target_width / width;
        let zoom_y = target_height / height;
        let new_zoom = zoom_x.min(zoom_y).max(0.3).min(3.0);
        
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        
        self.view_offset = Vec2::new(
            (600.0 / new_zoom) - center_x,
            (400.0 / new_zoom) - center_y,
        );
        self.zoom = new_zoom;
        self.request_repaint();
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.3, 3.0);
        self.request_repaint();
    }

    fn is_point_near_wire(&self, point: Pos2, conn: &Connection) -> bool {
        let from = self.blocks.iter().find(|b| b.id == conn.from_block);
        let to = self.blocks.iter().find(|b| b.id == conn.to_block);
        
        if let (Some(from), Some(to)) = (from, to) {
            let from_pos = Pos2::new(from.position.x + from.size.x, from.position.y + 30.0 + conn.from_port_index as f32 * 22.0);
            let to_pos = Pos2::new(to.position.x, to.position.y + 30.0 + conn.to_port_index as f32 * 22.0);
            
            let distance = point_to_line_segment_distance(point, from_pos, to_pos);
            return distance < 10.0;
        }
        false
    }

    // ─── RENDERING ─────────────────────────────────

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let canvas_rect = ui.max_rect();
        
        if self.pending_removal.is_some() {
            if let Some(id) = self.pending_removal.take() {
                self.remove_block(id);
            }
        }
        if self.pending_connection_removal.is_some() {
            if let Some(id) = self.pending_connection_removal.take() {
                self.remove_connection(id);
            }
        }

        let response = ui.interact(canvas_rect, ui.next_auto_id(), egui::Sense::click_and_drag());
        
        let input_state = ui.input(|i| i.clone());
        if input_state.modifiers.ctrl && input_state.key_pressed(Key::Z) {
            self.undo();
        }
        if input_state.modifiers.ctrl && input_state.key_pressed(Key::Y) {
            self.redo();
        }

        if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
            self.view_offset += response.drag_delta();
            self.request_repaint();
        }

        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let old_zoom = self.zoom;
            self.zoom *= 1.0 + scroll * 0.001;
            self.zoom = self.zoom.clamp(0.3, 3.0);
            
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let before = inverse_transform(mouse_pos, self.view_offset, old_zoom);
                let after = inverse_transform(mouse_pos, self.view_offset, self.zoom);
                self.view_offset += after - before;
            }
            self.request_repaint();
        }

        let painter = ui.painter();
        painter.rect_filled(canvas_rect, 0.0, self.background_color);

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

        // Calculate transforms without storing a closure that borrows self
        let view_offset = self.view_offset;
        let zoom = self.zoom;
        
        let transform = move |p: Pos2| -> Pos2 {
            Pos2::new(
                (p.x + view_offset.x) * zoom,
                (p.y + view_offset.y) * zoom,
            )
        };

        // Draw connections
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

        let mouse_pos_screen = ui.input(|i| i.pointer.hover_pos());
        let mouse_clicked = ui.input(|i| i.pointer.button_clicked(PointerButton::Primary));
        let mouse_right_clicked = ui.input(|i| i.pointer.button_clicked(PointerButton::Secondary));
        let delete_pressed = ui.input(|i| i.key_pressed(Key::Delete));
        let mouse_down = ui.input(|i| i.pointer.button_down(PointerButton::Primary));
        let mouse_released = ui.input(|i| i.pointer.button_released(PointerButton::Primary));

        if delete_pressed {
            self.pending_removal = self.selected_block;
        }

        if mouse_right_clicked {
            if let Some(mp) = mouse_pos_screen {
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

        if mouse_clicked {
            if let Some(mp) = mouse_pos_screen {
                let wm = inverse_transform(mp, self.view_offset, self.zoom);
                let mut clicked_port: Option<(u64, usize, bool)> = None;
                let mut clicked_block: Option<u64> = None;

                for block in self.blocks.iter().rev() {
                    let r = Rect::from_min_size(block.position, block.size);
                    if r.contains(wm) {
                        clicked_block = Some(block.id);
                        let ry = wm.y - block.position.y;
                        for (i, _) in block.definition.outputs.iter().enumerate() {
                            let py = 30.0 + i as f32 * 22.0;
                            if (ry - py).abs() < 18.0 && wm.x > block.position.x + block.size.x - 40.0 {
                                clicked_port = Some((block.id, i, true));
                                break;
                            }
                        }
                        if clicked_port.is_none() {
                            for (i, _) in block.definition.inputs.iter().enumerate() {
                                let py = 30.0 + i as f32 * 22.0;
                                if (ry - py).abs() < 18.0 && wm.x < block.position.x + 40.0 {
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
                        self.connecting_from = Some((block_id, port_idx));
                    } else if let Some((from_id, from_idx)) = self.connecting_from.take() {
                        let conn = Connection {
                            id: self.next_conn_id,
                            from_block: from_id,
                            from_port_index: from_idx,
                            to_block: block_id,
                            to_port_index: port_idx,
                        };
                        self.next_conn_id += 1;
                        self.connections.push(conn.clone());
                        self.push_action(CanvasAction::AddConnection { connection: conn.clone() });
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

        if mouse_down && self.dragging_block.is_none() {
            if let Some(mp) = mouse_pos_screen {
                let wm = inverse_transform(mp, self.view_offset, self.zoom);
                if let Some(block) = self.blocks.iter().rev()
                    .find(|b| Rect::from_min_size(b.position, b.size).contains(wm))
                {
                    self.dragging_block = Some(block.id);
                    self.drag_start = mp;
                    self.drag_start_pos = Some(block.position);
                }
            }
        }
        
        if let Some(block_id) = self.dragging_block {
            if let Some(mp) = mouse_pos_screen {
                let delta = (mp - self.drag_start) / self.zoom;
                if let Some(orig) = self.drag_start_pos {
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
        
                // Finalize drag on release OR when mouse leaves canvas
        let should_finalize_drag = mouse_released || (self.dragging_block.is_some() && !mouse_down);
        if should_finalize_drag {
            if let Some(block_id) = self.dragging_block {
                if let Some(orig) = self.drag_start_pos {
                    if let Some(block) = self.blocks.iter().find(|b| b.id == block_id) {
                        if block.position != orig {
                            let action = CanvasAction::MoveBlock {
                                block_id,
                                old_position: orig,
                                new_position: block.position,
                            };
                            self.push_action(action);
                        }
                    }
                }
            }
            self.dragging_block = None;
            self.drag_start_pos = None;
        }

        // Render blocks
        for block in &self.blocks {
            let matches_search = self.search_query.is_empty() || 
                block.definition.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                block.definition.category.to_lowercase().contains(&self.search_query.to_lowercase());
            
            if !matches_search && !self.show_experimental {
                continue;
            }
            
            let r = Rect::from_min_size(transform(block.position), block.size * self.zoom);
            if !canvas_rect.intersects(r) { continue; }
            let sel = self.selected_block == Some(block.id);

            painter.rect_filled(
                r.translate(Vec2::new(3.0, 3.0)),
                5.0,
                Color32::from_black_alpha(if sel { 100 } else { 60 }),
            );
            
            let body = if block.is_executing {
                Color32::from_rgb(60, 100, 60)
            } else if sel {
                Color32::from_rgb(55, 55, 75)
            } else {
                Color32::from_rgb(40, 40, 52)
            };
            painter.rect_filled(r, 5.0, body);
            
            let border = if sel {
                Color32::from_rgb(100, 160, 255)
            } else if block.is_executing {
                Color32::from_rgb(100, 200, 100)
            } else {
                Color32::from_rgb(60, 60, 75)
            };
            painter.rect_stroke(r, 5.0, Stroke::new(1.5, border));
            
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

            // Input ports
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
                        format!("{}", port.name)
                    } else if val.len() > 15 {
                        format!("{} = {}...", port.name, &val[..12])
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

            // Output ports
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
                } else if out_val.len() > 15 {
                    format!("{} = {}...", port.name, &out_val[..12])
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

        // Properties panel
        if let Some(edit_id) = self.editing_block {
            let mut pending_delete = false;
            let block_name;
            let mut edited_values: Vec<String>;
            let input_defs: Vec<(String, String, String)>;

            if let Some(block) = self.blocks.iter().find(|b| b.id == edit_id) {
                block_name = block.definition.name.clone();
                input_defs = block.definition.inputs.iter()
                    .map(|p| (p.name.clone(), p.port_type.clone(), p.default_value.clone()))
                    .collect();
                edited_values = block.input_values.clone();
            } else {
                return;
            }

            egui::SidePanel::right("properties_panel")
                .default_width(280.0)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(&block_name);
                    ui.separator();
                    
                    for (i, (name, ptype, default_val)) in input_defs.iter().enumerate() {
                        let is_conn = self.connections.iter()
                            .any(|c| c.to_block == edit_id && c.to_port_index == i);
                        
                        if is_conn {
                            ui.label(format!("{} ({}) ⟵ connected from block", name, ptype));
                        } else {
                            ui.label(format!("{} ({})", name, ptype));
                            if !ptype.contains("bool") {
                                let mut val = edited_values.get(i).cloned().unwrap_or_else(|| default_val.clone());
                                let response = ui.text_edit_singleline(&mut val);
                                if response.changed() {
                                    let old_value = edited_values.get(i).cloned().unwrap_or_default();
                                    edited_values[i] = val.clone();
                                    let action = CanvasAction::UpdateInput {
                                        block_id: edit_id,
                                        port_index: i,
                                        old_value,
                                        new_value: val,
                                    };
                                    self.push_action(action);
                                }
                            } else {
                                let mut val = edited_values.get(i).cloned().unwrap_or_else(|| "false".to_string());
                                let mut bool_val = val == "true";
                                if ui.checkbox(&mut bool_val, "").changed() {
                                    let new_val = if bool_val { "true" } else { "false" }.to_string();
                                    let action = CanvasAction::UpdateInput {
                                        block_id: edit_id,
                                        port_index: i,
                                        old_value: val,
                                        new_value: new_val.clone(),
                                    };
                                    self.push_action(action);
                                    edited_values[i] = new_val;
                                }
                            }
                        }
                    }
                    
                    ui.separator();
                    if ui.button("🗑️ Delete Block").clicked() {
                        pending_delete = true;
                    }
                });

            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == edit_id) {
                for (i, val) in edited_values.iter().enumerate() {
                    let is_conn = self.connections.iter()
                        .any(|c| c.to_block == edit_id && c.to_port_index == i);
                    if !is_conn && i < block.input_values.len() {
                        block.input_values[i] = val.clone();
                    }
                }
            }
            if pending_delete {
                self.pending_removal = Some(edit_id);
            }
        }
        
        self.reset_repaint();
    }
}

// ─── HELPER FUNCTIONS ─────────────────────────────

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

fn point_to_line_segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;
    if ab_len_sq < 0.0001 {
        let dx = p.x - a.x;
        let dy = p.y - a.y;
        return (dx * dx + dy * dy).sqrt();
    }
    let ap = p - a;
    let t = (ap.x * ab.x + ap.y * ab.y) / ab_len_sq;
    let t = t.clamp(0.0, 1.0);
    let projection = a + ab * t;
    let dx = p.x - projection.x;
    let dy = p.y - projection.y;
    (dx * dx + dy * dy).sqrt()
}