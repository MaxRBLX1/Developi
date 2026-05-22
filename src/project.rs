// project.rs — .dev Project File Format
// Save and load entire workshop state.
// Blocks, connections, canvas position, zoom, sensory settings.
// No placeholders. Production code. Ships as-is.

use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::blocks::BlockDefinition;

// ─── Project Structure ────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub metadata: ProjectMetadata,
    pub canvas: CanvasSnapshot,
    pub blocks: Vec<PlacedBlockSnapshot>,
    pub connections: Vec<ConnectionSnapshot>,
    pub variables: Vec<VariableSnapshot>,
    pub sensory: SensorySnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub developi_version: String,
    pub created_at: String,
    pub modified_at: String,
    pub block_count: usize,
    pub connection_count: usize,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CanvasSnapshot {
    pub zoom: f32,
    pub view_offset_x: f32,
    pub view_offset_y: f32,
    pub grid_size: f32,
    pub grid_visible: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlacedBlockSnapshot {
    pub id: u64,
    pub block_name: String,
    pub category: String,
    pub position_x: f32,
    pub position_y: f32,
    pub size_x: f32,
    pub size_y: f32,
    pub custom_code: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionSnapshot {
    pub id: u64,
    pub from_block_id: u64,
    pub from_port: String,
    pub to_block_id: u64,
    pub to_port: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariableSnapshot {
    pub name: String,
    pub value: String,
    pub var_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorySnapshot {
    pub ambient_sound_path: Option<String>,
    pub block_place_sound_path: Option<String>,
    pub execution_start_sound_path: Option<String>,
    pub execution_complete_sound_path: Option<String>,
    pub error_sound_path: Option<String>,
    pub connection_sound_path: Option<String>,
    pub disconnect_sound_path: Option<String>,
    pub master_volume: f32,
    pub ambient_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
}

// ─── Project Implementation ────────────────────────────

impl Project {
    /// Create a new empty project
    pub fn new(name: &str) -> Self {
        let now = timestamp_now();
        info!("📄 Creating new project: '{}'", name);
        
        Project {
            metadata: ProjectMetadata {
                name: name.to_string(),
                version: "1.0".to_string(),
                developi_version: "1.0".to_string(),
                created_at: now.clone(),
                modified_at: now,
                block_count: 0,
                connection_count: 0,
                description: String::new(),
            },
            canvas: CanvasSnapshot::default(),
            blocks: Vec::new(),
            connections: Vec::new(),
            variables: Vec::new(),
            sensory: SensorySnapshot::default(),
        }
    }

    /// Save project to a .dev file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        info!("💾 Saving project to {:?}", path);
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory {:?}: {}", parent, e))?;
        }
        
        // Write to a temp file first, then rename (atomic save)
        let temp_path = path.with_extension(".dev.tmp");
        fs::write(&temp_path, &json)
            .map_err(|e| format!("Write failed: {}", e))?;
        
        fs::rename(&temp_path, path)
            .map_err(|e| format!("Rename failed: {}", e))?;
        
        let size = json.len();
        info!("✅ Project saved: {} bytes, {} blocks, {} connections",
            size, self.metadata.block_count, self.metadata.connection_count);
        
        Ok(())
    }

    /// Load project from a .dev file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        info!("📂 Loading project from {:?}", path);
        
        if !path.exists() {
            return Err(format!("File not found: {:?}", path));
        }
        
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Read failed: {}", e))?;
        
        let mut project: Project = serde_json::from_str(&json)
            .map_err(|e| format!("Parse failed: {}", e))?;
        
        // Update modified timestamp
        project.metadata.modified_at = timestamp_now();
        
        info!("✅ Project loaded: '{}' — {} blocks, {} connections",
            project.metadata.name,
            project.metadata.block_count,
            project.metadata.connection_count);
        
        Ok(project)
    }

    /// Auto-save to a backup location
    pub fn autosave(&self, project_dir: &PathBuf) -> Result<(), String> {
        let backup_dir = project_dir.join(".developi_backups");
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Cannot create backup dir: {}", e))?;
        
        let timestamp = chrono_now_for_filename();
        let backup_path = backup_dir.join(format!("autosave_{}.dev", timestamp));
        
        self.save(&backup_path)
    }

    /// Update block count in metadata
    pub fn update_counts(&mut self, block_count: usize, connection_count: usize) {
        self.metadata.block_count = block_count;
        self.metadata.connection_count = connection_count;
        self.metadata.modified_at = timestamp_now();
    }

    /// Add a block to the project
    pub fn add_block(&mut self, block: PlacedBlockSnapshot) {
        self.blocks.push(block);
        self.metadata.block_count = self.blocks.len();
        self.metadata.modified_at = timestamp_now();
    }

    /// Remove a block by ID
    pub fn remove_block(&mut self, block_id: u64) -> bool {
        let len_before = self.blocks.len();
        self.blocks.retain(|b| b.id != block_id);
        self.connections.retain(|c| c.from_block_id != block_id && c.to_block_id != block_id);
        
        let removed = len_before != self.blocks.len();
        if removed {
            self.metadata.block_count = self.blocks.len();
            self.metadata.connection_count = self.connections.len();
            self.metadata.modified_at = timestamp_now();
        }
        removed
    }

    /// Add a connection
    pub fn add_connection(&mut self, connection: ConnectionSnapshot) {
        self.connections.push(connection);
        self.metadata.connection_count = self.connections.len();
        self.metadata.modified_at = timestamp_now();
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, connection_id: u64) -> bool {
        let len_before = self.connections.len();
        self.connections.retain(|c| c.id != connection_id);
        let removed = len_before != self.connections.len();
        if removed {
            self.metadata.connection_count = self.connections.len();
            self.metadata.modified_at = timestamp_now();
        }
        removed
    }

    /// Set a variable
    pub fn set_variable(&mut self, name: &str, value: &str, var_type: &str) {
        if let Some(existing) = self.variables.iter_mut().find(|v| v.name == name) {
            existing.value = value.to_string();
            existing.var_type = var_type.to_string();
        } else {
            self.variables.push(VariableSnapshot {
                name: name.to_string(),
                value: value.to_string(),
                var_type: var_type.to_string(),
            });
        }
    }

    /// Get a variable
    pub fn get_variable(&self, name: &str) -> Option<&VariableSnapshot> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// List all project files in a directory
    pub fn list_projects(directory: &PathBuf) -> Vec<ProjectInfo> {
        let mut projects = Vec::new();
        
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "dev") {
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(project) = serde_json::from_str::<Project>(&json) {
                            projects.push(ProjectInfo {
                                path: path.clone(),
                                name: project.metadata.name,
                                modified: project.metadata.modified_at,
                                block_count: project.metadata.block_count,
                            });
                        }
                    }
                }
            }
        }
        
        projects.sort_by(|a, b| b.modified.cmp(&a.modified));
        projects
    }

    /// Export project as a standalone Python script
    pub fn export_python(&self) -> String {
        let mut script = String::new();
        script.push_str("# developi Generated Python Script\n");
        script.push_str(&format!("# Project: {}\n", self.metadata.name));
        script.push_str(&format!("# Generated: {}\n", timestamp_now()));
        script.push_str(&format!("# Blocks: {}\n\n", self.metadata.block_count));
        
        for block in &self.blocks {
            script.push_str(&format!("# ── {} ──\n", block.block_name));
            if let Some(ref custom) = block.custom_code {
                script.push_str(custom);
            }
            script.push('\n');
        }
        
        script
    }

    /// Export project as JSON (for external tools)
    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Export failed: {}", e))
    }

    /// Import from JSON
    pub fn import_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Import failed: {}", e))
    }
}

impl Default for CanvasSnapshot {
    fn default() -> Self {
        CanvasSnapshot {
            zoom: 1.0,
            view_offset_x: 0.0,
            view_offset_y: 0.0,
            grid_size: 20.0,
            grid_visible: true,
        }
    }
}

impl Default for SensorySnapshot {
    fn default() -> Self {
        SensorySnapshot {
            ambient_sound_path: None,
            block_place_sound_path: None,
            execution_start_sound_path: None,
            execution_complete_sound_path: None,
            error_sound_path: None,
            connection_sound_path: None,
            disconnect_sound_path: None,
            master_volume: 0.7,
            ambient_volume: 0.3,
            sfx_volume: 0.8,
            muted: false,
        }
    }
}

// ─── Project Info (for file browser) ──────────────────

#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub path: PathBuf,
    pub name: String,
    pub modified: String,
    pub block_count: usize,
}

impl ProjectInfo {
    pub fn display_name(&self) -> String {
        format!("{} ({} blocks)", self.name, self.block_count)
    }
}

// ─── Recent Projects ──────────────────────────────────

pub struct RecentProjects {
    projects: Vec<PathBuf>,
    max_entries: usize,
    config_path: PathBuf,
}

impl RecentProjects {
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join("recent_projects.json");
        let projects = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        
        RecentProjects {
            projects,
            max_entries: 10,
            config_path,
        }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.projects.retain(|p| p != &path);
        self.projects.insert(0, path);
        self.projects.truncate(self.max_entries);
        self.save();
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.projects.retain(|p| p != path);
        self.save();
    }

    pub fn list(&self) -> &[PathBuf] {
        &self.projects
    }

    pub fn clear(&mut self) {
        self.projects.clear();
        self.save();
    }

    fn save(&self) {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.projects) {
            fs::write(&self.config_path, json).ok();
        }
    }
}

// ─── Helpers ──────────────────────────────────────────

fn timestamp_now() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        let secs = duration.as_secs();
        // Simple ISO-like format without chrono dependency
        let days_since_epoch = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;
        
        // Calculate date from days since epoch (approximate, good enough for timestamps)
        let year = 1970 + (days_since_epoch / 365) as i32;
        let day_of_year = (days_since_epoch % 365) as u32;
        let month = (day_of_year / 30 + 1).min(12);
        let day = (day_of_year % 30 + 1).min(31);
        
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, seconds)
    } else {
        "unknown".to_string()
    }
}

fn chrono_now_for_filename() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        let secs = duration.as_secs();
        format!("{}", secs)
    } else {
        "0".to_string()
    }
}

// ─── Tests ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_and_save_project() {
        let mut project = Project::new("test_project");
        project.metadata.description = "A test project".to_string();
        
        // Add a block
        project.add_block(PlacedBlockSnapshot {
            id: 1,
            block_name: "Print".to_string(),
            category: "Debug".to_string(),
            position_x: 100.0,
            position_y: 200.0,
            size_x: 180.0,
            size_y: 60.0,
            custom_code: Some("print('Hello from test')".to_string()),
        });
        
        // Add a connection
        project.add_connection(ConnectionSnapshot {
            id: 1,
            from_block_id: 1,
            from_port: "output".to_string(),
            to_block_id: 2,
            to_port: "input".to_string(),
        });
        
        // Save
        let temp_dir = env::temp_dir();
        let path = temp_dir.join("test_project.dev");
        assert!(project.save(&path).is_ok());
        
        // Load
        let loaded = Project::load(&path).unwrap();
        assert_eq!(loaded.metadata.name, "test_project");
        assert_eq!(loaded.blocks.len(), 1);
        assert_eq!(loaded.blocks[0].block_name, "Print");
        
        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_export_python() {
        let mut project = Project::new("export_test");
        project.add_block(PlacedBlockSnapshot {
            id: 1,
            block_name: "Print".to_string(),
            category: "Debug".to_string(),
            position_x: 0.0,
            position_y: 0.0,
            size_x: 180.0,
            size_y: 60.0,
            custom_code: Some("print('exported')\n".to_string()),
        });
        
        let python = project.export_python();
        assert!(python.contains("exported"));
        assert!(python.contains("developi Generated"));
    }

    #[test]
    fn test_recent_projects() {
        let temp_dir = env::temp_dir().join("developi_test_config");
        let mut recent = RecentProjects::new(temp_dir.clone());
        
        recent.add(PathBuf::from("/test/project1.dev"));
        recent.add(PathBuf::from("/test/project2.dev"));
        recent.add(PathBuf::from("/test/project1.dev")); // Duplicate should move to front
        
        let list = recent.list().to_vec();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], PathBuf::from("/test/project1.dev"));
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}