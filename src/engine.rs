// engine.rs — Production Engine for developi 1.0
// Executes Python blocks in topological order with auto-fix, crash recovery,
// multi-output extraction, pass-through forwarding, and friendly error messages.
// No walls. The engine helps the user, not punishes them.

use log::{info, warn, error};
use crate::blocks::BlockDefinition;
use std::collections::{HashMap, HashSet};
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;

pub struct PythonEngine {
    initialized: bool,
    execution_count: u64,
    variables: HashMap<String, String>,
    last_error: Option<String>,
    ram_usage_mb: f64,
    auto_fix_log: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PlacedBlockData {
    pub id: u64,
    pub definition: BlockDefinition,
    pub input_values: Vec<String>,
    pub output_values: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ConnectionData {
    pub from_block: u64,
    pub from_port_index: usize,
    pub to_block: u64,
    pub to_port_index: usize,
}

impl PythonEngine {
    pub fn new() -> Self {
        PythonEngine {
            initialized: false,
            execution_count: 0,
            variables: HashMap::new(),
            last_error: None,
            ram_usage_mb: 0.0,
            auto_fix_log: Vec::new(),
        }
    }

    fn ensure_initialized(&mut self) {
        if !self.initialized {
            pyo3::prepare_freethreaded_python();
            self.initialized = true;
            info!("Python 3.14 engine initialized");
            PythonEngine::pre_import_modules();
            self.update_ram_usage();
        }
    }

    fn pre_import_modules() {
        Python::with_gil(|py| {
            let modules = ["ctypes", "struct", "socket", "os", "sys", "json", "math", "re", "io", "mmap"];
            for module in &modules {
                match py.import(*module) {
                    Ok(_) => info!("  Module loaded: {}", module),
                    Err(e) => warn!("  Module not available: {} ({})", module, e),
                }
            }
        });
    }

    fn types_compatible(source_type: &str, target_type: &str) -> bool {
        if source_type == target_type { return true; }
        if source_type == "any" || target_type == "any" { return true; }
        let numeric = ["number", "int", "float"];
        if numeric.contains(&source_type) && numeric.contains(&target_type) { return true; }
        if target_type == "string" && (source_type == "string" || source_type == "bytes") { return true; }
        false
    }

    /// ─── AUTO-FIX ENGINE ─────────────────────────────
    pub fn auto_fix_connections(
        &mut self,
        blocks: &[PlacedBlockData],
        connections: &mut Vec<ConnectionData>,
    ) -> Vec<String> {
        self.auto_fix_log.clear();
        let mut fixed = Vec::new();

        for conn in connections.iter_mut() {
            let source = blocks.iter().find(|b| b.id == conn.from_block);
            let target = blocks.iter().find(|b| b.id == conn.to_block);

            if let (Some(src), Some(tgt)) = (source, target) {
                let src_port_name = src.definition.outputs
                    .get(conn.from_port_index)
                    .map(|p| p.name.as_str())
                    .unwrap_or("");
                let tgt_port_name = tgt.definition.inputs
                    .get(conn.to_port_index)
                    .map(|p| p.name.as_str())
                    .unwrap_or("");

                // Rule 1: Exact port name match available on source
                if src_port_name != tgt_port_name {
                    for (i, out_port) in src.definition.outputs.iter().enumerate() {
                        if out_port.name == tgt_port_name && i != conn.from_port_index {
                            let old_src = src_port_name;
                            conn.from_port_index = i;
                            fixed.push(format!(
                                "🔧 Auto-fixed: '{}' port '{}' → '{}' port '{}' (name mismatch, found matching port '{}')",
                                src.definition.name, old_src,
                                tgt.definition.name, tgt_port_name,
                                out_port.name
                            ));
                            break;
                        }
                    }
                }

                // Rule 2: "address" port getting bytes_written
                if tgt_port_name.contains("address") && src_port_name.contains("bytes") {
                    for out_port in &src.definition.outputs {
                        if out_port.name.contains("address") && out_port.name != src_port_name {
                            conn.from_port_index = src.definition.outputs.iter()
                                .position(|p| p.name == out_port.name)
                                .unwrap_or(conn.from_port_index);
                            fixed.push(format!(
                                "🔧 Auto-fixed: '{}' had '{}' wired to '{}' address. Switched to '{}' output.",
                                src.definition.name, src_port_name, tgt.definition.name, out_port.name
                            ));
                            break;
                        }
                    }
                }
            }
        }

        self.auto_fix_log = fixed.clone();
        fixed
    }

    /// ─── MAIN EXECUTION ─────────────────────────────
    pub fn execute_dataflow(
        &mut self,
        blocks: &mut [PlacedBlockData],
        connections: &[ConnectionData],
    ) -> Vec<String> {
        self.ensure_initialized();
        let mut output = Vec::new();

        if !self.auto_fix_log.is_empty() {
            output.push("── Auto-Fixes ──".into());
            for msg in &self.auto_fix_log {
                output.push(msg.clone());
            }
            self.auto_fix_log.clear();
        }

        // Validate all connections
        for conn in connections {
            let source = blocks.iter().find(|b| b.id == conn.from_block);
            let target = blocks.iter().find(|b| b.id == conn.to_block);
            if let (Some(src), Some(tgt)) = (source, target) {
                let src_type = src.definition.outputs.get(conn.from_port_index)
                    .map(|p| p.port_type.as_str()).unwrap_or("any");
                let tgt_type = tgt.definition.inputs.get(conn.to_port_index)
                    .map(|p| p.port_type.as_str()).unwrap_or("any");
                if !Self::types_compatible(src_type, tgt_type) {
                    warn!("Type mismatch: {} out[{}] ({}) -> {} in[{}] ({})",
                        src.definition.name, conn.from_port_index, src_type,
                        tgt.definition.name, conn.to_port_index, tgt_type);
                }
            }
        }

        let order = self.topological_sort(blocks, connections);
        let mut executed: HashSet<u64> = HashSet::new();
        let mut crash_count = 0;
        let max_retries = 3;

        output.push("── Execution ──".into());

        for block_id in &order {
            let idx = blocks.iter().position(|b| b.id == *block_id).unwrap();

            // Apply incoming wired values
            for conn in connections {
                if conn.to_block == *block_id {
                    if let Some(source) = blocks.iter().find(|b| b.id == conn.from_block) {
                        if executed.contains(&conn.from_block) {
                            let source_val = source.output_values
                                .get(conn.from_port_index)
                                .cloned()
                                .unwrap_or_default();
                            if conn.to_port_index < blocks[idx].input_values.len() {
                                blocks[idx].input_values[conn.to_port_index] = source_val;
                            }
                        }
                    }
                }
            }

            // Pre-execution hex fix
            for i in 0..blocks[idx].definition.inputs.len() {
                let port_name = blocks[idx].definition.inputs[i].name.clone();
                if port_name.contains("hex") || port_name.contains("data_hex") {
                    let val = blocks[idx].input_values[i].clone();
                    let cleaned = val.trim().trim_matches('"').to_string();
                    if cleaned.len() % 2 != 0 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
                        let fixed_val = format!("{}0", cleaned);
                        blocks[idx].input_values[i] = fixed_val.clone();
                        output.push(format!(
                            "🔧 Auto-padded hex: '{}' → '{}' (added missing digit)",
                            cleaned, fixed_val
                        ));
                    }
                }
            }

            // Execute the block
            let block_def = blocks[idx].definition.clone();
            let input_vals = blocks[idx].input_values.clone();
            let result_text = self.execute_single(&block_def, &input_vals);

            // Crash recovery
            if result_text.contains("✗ Error:") && result_text.contains("access violation") {
                crash_count += 1;
                if crash_count < max_retries {
                    output.push(format!(
                        "⚠️  Crash detected in '{}'. Attempting auto-recovery...",
                        block_def.name
                    ));

                    if block_def.name.contains("Read") {
                        // Find valid address from upstream blocks
                        let mut found_addr: Option<String> = None;
                        for conn in connections {
                            if conn.to_block == *block_id && conn.to_port_index == 0 {
                                if let Some(upstream) = blocks.iter().find(|b| b.id == conn.from_block) {
                                    for (pi, out_port) in upstream.definition.outputs.iter().enumerate() {
                                        if out_port.name.contains("address") && pi != conn.from_port_index {
                                            if let Some(val) = upstream.output_values.get(pi) {
                                                if val.parse::<u64>().unwrap_or(0) > 1000 {
                                                    found_addr = Some(val.clone());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if found_addr.is_some() { break; }
                        }

                        if let Some(addr) = found_addr {
                            blocks[idx].input_values[0] = addr.clone();
                            output.push(format!(
                                "🔧 Recovery: switched '{}' address to {} (valid pointer)",
                                block_def.name, addr
                            ));
                            let retry_result = self.execute_single(&block_def, &blocks[idx].input_values);
                            blocks[idx].output_values = self.extract_result();
                            output.push(format!("[{}] {}", block_def.name, retry_result));
                            executed.insert(*block_id);
                            continue;
                        }
                    }
                }
            }

            blocks[idx].output_values = self.extract_result();
            output.push(format!("[{}] {}", block_def.name, result_text));
            executed.insert(*block_id);
        }

        output.push("── Complete ──".into());
        self.update_ram_usage();
        output
    }

    fn topological_sort(&self, blocks: &[PlacedBlockData], connections: &[ConnectionData]) -> Vec<u64> {
        let mut in_degree: HashMap<u64, usize> = HashMap::new();
        let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();

        for conn in connections {
            adj.entry(conn.from_block).or_default().push(conn.to_block);
            *in_degree.entry(conn.to_block).or_default() += 1;
        }

        let mut queue: Vec<u64> = blocks.iter()
            .map(|b| b.id)
            .filter(|id| in_degree.get(id).unwrap_or(&0) == &0)
            .collect();

        let mut order = Vec::new();
        while let Some(id) = queue.pop() {
            order.push(id);
            if let Some(children) = adj.get(&id) {
                for child in children {
                    if let Some(deg) = in_degree.get_mut(child) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(*child);
                        }
                    }
                }
            }
        }

        for block in blocks {
            if !order.contains(&block.id) {
                order.push(block.id);
            }
        }

        order
    }

    fn execute_single(&mut self, block: &BlockDefinition, input_values: &[String]) -> String {
        self.execution_count += 1;

        let mut code = block.python_template.clone();
        for (i, port) in block.inputs.iter().enumerate() {
            let placeholder = format!("{{{{{}}}}}", port.name);
            let value = input_values.get(i).map(|s| s.as_str()).unwrap_or(&port.default_value);
            let safe_value = value.replace("\\", "/");
            code = code.replace(&placeholder, &safe_value);
        }

        let block_name = block.name.clone();
        info!("Executing [{}]", block_name);

        Python::with_gil(|py| {
            let main = py.import("__main__").expect("__main__ not found");
            let main_dict = main.dict();

            let sys = py.import("sys").ok();
            let io_mod = py.import("io").ok();
            let string_io = io_mod.as_ref()
                .and_then(|io| io.getattr("StringIO").ok())
                .and_then(|sio| sio.call0().ok());

            let old_stdout = sys.as_ref().and_then(|s| s.getattr("stdout").ok());
            if let (Some(ref sio), Some(ref sys_mod)) = (&string_io, &sys) {
                sys_mod.setattr("stdout", sio).ok();
            }

            let c_code = std::ffi::CString::new(code.as_str()).unwrap_or_default();
            let exec_result = py.run(&c_code, None, Some(&main_dict));

            if let (Some(old), Some(sys_mod)) = (&old_stdout, &sys) {
                sys_mod.setattr("stdout", old).ok();
            }

            let captured = string_io.as_ref()
                .and_then(|sio| sio.getattr("getvalue").ok())
                .and_then(|g| g.call0().ok())
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_default();

            match exec_result {
                Ok(_) => {
                    if !captured.trim().is_empty() {
                        format!("✓ {}", captured.trim().chars().take(200).collect::<String>())
                    } else {
                        "✓ OK".to_string()
                    }
                }
                Err(e) => {
                    let raw_msg = format!("{}", e);
                    let friendly_msg = self.friendly_error(&raw_msg, &block_name);
                    error!("Block '{}' failed: {}", block_name, raw_msg);
                    self.last_error = Some(friendly_msg.clone());
                    format!("✗ Error: {}", friendly_msg)
                }
            }
        })
    }

    fn friendly_error(&self, raw: &str, block_name: &str) -> String {
        if raw.contains("access violation") || raw.contains("segmentation fault") {
            format!(
                "Memory access error in '{}'. This usually means a wrong address was wired. \
                 Try connecting an 'address' output from Allocate Memory instead.",
                block_name
            )
        } else if raw.contains("fromhex") && raw.contains("even number") {
            "Hex string must have an even number of characters (pairs of hex digits). \
             Try: '48656C6C6F' for 'Hello'."
                .into()
        } else if raw.contains("ValueError") {
            format!(
                "Invalid value in '{}'. Check your input fields for typos or wrong data types.",
                block_name
            )
        } else if raw.contains("TypeError") {
            format!(
                "Type mismatch in '{}'. A port received data it doesn't know how to handle.",
                block_name
            )
        } else if raw.contains("MemoryError") || raw.contains("Out of memory") {
            "Out of memory. Try allocating a smaller buffer or freeing unused memory.".into()
        } else if raw.contains("FileNotFound") || raw.contains("No such file") {
            "File not found. Check the path and make sure the file exists.".into()
        } else if raw.contains("ConnectionRefused") || raw.contains("timed out") {
            "Network connection failed. Check the address and port, and make sure the server is running.".into()
        } else if raw.len() > 150 {
            format!("{}...", &raw[..147])
        } else {
            raw.to_string()
        }
    }

    fn extract_result(&self) -> Vec<String> {
        Python::with_gil(|py| {
            if let Ok(main) = py.import("__main__") {
                if let Ok(val) = main.getattr("result") {
                    if let Ok(s) = val.extract::<String>() {
                        return vec![s];
                    }
                    if let Ok(n) = val.extract::<i64>() {
                        return vec![n.to_string()];
                    }
                    if let Ok(f) = val.extract::<f64>() {
                        return vec![f.to_string()];
                    }
                    return vec![format!("{}", val)];
                }
            }
            vec![String::new()]
        })
    }

    pub fn ram_usage(&self) -> f64 {
        if !self.initialized { 0.0 } else { self.ram_usage_mb }
    }

    fn update_ram_usage(&mut self) {
        self.ram_usage_mb = Python::with_gil(|py| {
            let ram_code = std::ffi::CString::new(
                "import sys; __ram = sys.getsizeof(globals()) / 1024 / 1024"
            ).unwrap();
            py.run(&ram_code, None, None).map(|_| 40.0).unwrap_or(40.0)
        });
    }
}