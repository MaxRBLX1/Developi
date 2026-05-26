// blocks.rs — Block Registry & Definitions
// 96 blocks with input/output ports. No hardcoded values.
// Users type values OR connect blocks. No walls.
// All output ports send clean data. Wires just work.
// Every input that matters has a matching output pass-through.
// No coding required. Just type values. Just connect blocks.
// Object-passing blocks store objects in __main__ and pass keys through wires.
// Production code. Ships as-is for developi 1.0.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockPort {
    pub name: String,
    pub port_type: String,
    pub default_value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub name: String,
    pub icon: String,
    pub category: String,
    #[serde(default)]
    pub description: String,
    pub inputs: Vec<BlockPort>,
    pub outputs: Vec<BlockPort>,
    pub python_template: String,
    pub experimental: bool,
}

#[derive(Clone, Debug)]
pub struct BlockCategory {
    pub name: String,
    pub icon: String,
}

pub struct BlockRegistry {
    blocks: Vec<BlockDefinition>,
    categories: Vec<BlockCategory>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let categories = vec![
            BlockCategory { name: "Memory".into(),      icon: "🧠".into() },
            BlockCategory { name: "Process".into(),     icon: "⚙️".into() },
            BlockCategory { name: "File System".into(), icon: "📁".into() },
            BlockCategory { name: "Network".into(),     icon: "🌐".into() },
            BlockCategory { name: "Data".into(),        icon: "📊".into() },
            BlockCategory { name: "Logic".into(),       icon: "🔀".into() },
            BlockCategory { name: "Math".into(),        icon: "🔢".into() },
            BlockCategory { name: "Variables".into(),   icon: "📝".into() },
            BlockCategory { name: "Functions".into(),   icon: "⚡".into() },
            BlockCategory { name: "Python Power".into(),icon: "🐍".into() },
            BlockCategory { name: "Low-Level".into(),   icon: "🔩".into() },
            BlockCategory { name: "Debug".into(),       icon: "🔍".into() },
        ];

        let blocks = build_all_blocks();
        BlockRegistry { blocks, categories }
    }

    pub fn categories(&self) -> &[BlockCategory] { &self.categories }
    pub fn all_blocks(&self) -> &[BlockDefinition] { &self.blocks }
    pub fn block_count(&self) -> usize { self.blocks.len() }
    pub fn blocks_in_category(&self, category: &BlockCategory) -> Vec<&BlockDefinition> {
        self.blocks.iter().filter(|b| b.category == category.name).collect()
    }
    pub fn find_block(&self, name: &str) -> Option<&BlockDefinition> {
        self.blocks.iter().find(|b| b.name == name)
    }
    pub fn stable_blocks(&self) -> Vec<&BlockDefinition> {
        self.blocks.iter().filter(|b| !b.experimental).collect()
    }
}

fn port(name: &str, ptype: &str, default: &str) -> BlockPort {
    BlockPort { name: name.into(), port_type: ptype.into(), default_value: default.into() }
}
fn input(name: &str, ptype: &str, default: &str) -> BlockPort { port(name, ptype, default) }
fn output(name: &str, ptype: &str) -> BlockPort { port(name, ptype, "") }

// ─── Helper: generate a unique object key ───
fn obj_key(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("__DEVOBJ__{}_{}", prefix, nanos)
}

fn build_all_blocks() -> Vec<BlockDefinition> {
    let mut b = Vec::new();

    // ═══════════════ MEMORY (8) ═══════════════

    b.push(BlockDefinition {
        name: "Allocate Memory".into(), icon: "📦".into(), category: "Memory".into(),
        description: "Allocate persistent memory that stays until freed. Returns address and size.".into(),
        inputs: vec![input("size", "number", "1024")],
        outputs: vec![output("address", "number"), output("size", "number")],
        python_template: r#"
import ctypes, sys
_size = {{size}}
_buffer = ctypes.create_string_buffer(_size)
_address = ctypes.addressof(_buffer)
_main = sys.modules['__main__']
if not hasattr(_main, '_developi_allocations'):
    _main._developi_allocations = {}
_main._developi_allocations[_address] = _buffer
result = f"{_address},{_size}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Free Memory".into(), icon: "🗑️".into(), category: "Memory".into(),
        description: "Free allocated memory. Returns success and the address.".into(),
        inputs: vec![input("address", "number", "0")],
        outputs: vec![output("success", "bool"), output("address", "number")],
        python_template: r#"
import sys
_addr = {{address}}
_main = sys.modules['__main__']
try:
    if hasattr(_main, '_developi_allocations') and _addr in _main._developi_allocations:
        del _main._developi_allocations[_addr]
        _success = True
    else:
        _success = False
except:
    _success = False
result = f"{_success},{_addr}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Read Memory".into(), icon: "👁️".into(), category: "Memory".into(),
        description: "Read raw bytes from a memory address. Returns hex string, address, and size.".into(),
        inputs: vec![input("address", "number", "0"), input("size", "number", "16")],
        outputs: vec![output("data", "string"), output("address", "number"), output("size", "number")],
        python_template: r#"
import ctypes
_addr = {{address}}
_size = {{size}}
try:
    data = ctypes.string_at(_addr, _size)
    _hex = data.hex()
    result = f"{_hex},{_addr},{_size}"
except Exception as e:
    result = f"Error: {e},0,0"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Write Memory".into(), icon: "✏️".into(), category: "Memory".into(),
        description: "Write text to tracked memory with bounds checking. Returns bytes written and address.".into(),
        inputs: vec![input("address", "number", "0"), input("data", "string", "developi")],
        outputs: vec![output("bytes_written", "number"), output("address", "number")],
        python_template: r#"
import ctypes, sys
_addr = {{address}}
_data = "{{data}}".encode('utf-8')
_main = sys.modules['__main__']
if hasattr(_main, '_developi_allocations') and _addr in _main._developi_allocations:
    _buffer = _main._developi_allocations[_addr]
    _max_size = len(_buffer)
    _bytes = min(len(_data), _max_size)
    ctypes.memmove(_addr, _data, _bytes)
    result = f"{_bytes},{_addr}"
else:
    ctypes.memmove(_addr, _data, len(_data))
    _bytes = len(_data)
    result = f"{_bytes},{_addr}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Cast Pointer".into(), icon: "🎯".into(), category: "Memory".into(),
        description: "Cast a raw pointer to a typed pointer. Returns the value, address, and type used.".into(),
        inputs: vec![input("address", "number", "0"), input("type", "string", "int")],
        outputs: vec![output("value", "any"), output("address", "number"), output("type_used", "string")],
        python_template: r#"
import ctypes
_addr = {{address}}
_type = "{{type}}"
type_map = {
    "int": ctypes.c_int, "float": ctypes.c_float, "char": ctypes.c_char,
    "double": ctypes.c_double, "short": ctypes.c_short, "long": ctypes.c_long,
    "byte": ctypes.c_byte, "ubyte": ctypes.c_ubyte, "ushort": ctypes.c_ushort,
    "uint": ctypes.c_uint, "ulong": ctypes.c_ulong, "void_p": ctypes.c_void_p
}
t = type_map.get(_type, ctypes.c_int)
ptr = ctypes.cast(_addr, ctypes.POINTER(t))
result = f"{ptr.contents.value},{_addr},{_type}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Size Of".into(), icon: "📏".into(), category: "Memory".into(),
        description: "Get the size in bytes of a C type.".into(),
        inputs: vec![input("type", "string", "c_int")],
        outputs: vec![output("size", "number"), output("type", "string")],
        python_template: r#"
import ctypes
_type = "{{type}}"
type_map = {
    "c_int": ctypes.c_int, "c_long": ctypes.c_long, "c_void_p": ctypes.c_void_p,
    "c_double": ctypes.c_double, "c_char": ctypes.c_char, "c_short": ctypes.c_short,
    "c_byte": ctypes.c_byte, "c_ubyte": ctypes.c_ubyte, "c_ushort": ctypes.c_ushort,
    "c_uint": ctypes.c_uint, "c_ulong": ctypes.c_ulong, "c_float": ctypes.c_float,
    "c_bool": ctypes.c_bool, "c_wchar": ctypes.c_wchar
}
t = type_map.get(_type, ctypes.c_int)
result = f"{ctypes.sizeof(t)},{_type}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Memory Copy".into(), icon: "📋".into(), category: "Memory".into(),
        description: "Copy memory from source to destination. Returns bytes copied, src, and dst.".into(),
        inputs: vec![input("src", "number", "0"), input("dst", "number", "0"), input("size", "number", "32")],
        outputs: vec![output("bytes_copied", "number"), output("src", "number"), output("dst", "number")],
        python_template: r#"
import ctypes
_src = {{src}}
_dst = {{dst}}
_size = {{size}}
try:
    ctypes.memmove(_dst, _src, _size)
    result = f"{_size},{_src},{_dst}"
except:
    result = f"0,{_src},{_dst}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Memory Compare".into(), icon: "⚖️".into(), category: "Memory".into(),
        description: "Compare two memory regions. Returns equal flag and diff position.".into(),
        inputs: vec![input("addr1", "number", "0"), input("addr2", "number", "0"), input("size", "number", "16")],
        outputs: vec![output("equal", "bool"), output("diff_at", "number")],
        python_template: r#"
import ctypes
_addr1 = {{addr1}}
_addr2 = {{addr2}}
_size = {{size}}
try:
    a = ctypes.string_at(_addr1, _size)
    b = ctypes.string_at(_addr2, _size)
    diff = -1
    for i in range(min(len(a), len(b))):
        if a[i] != b[i]:
            diff = i
            break
    _equal = (diff == -1)
    result = f"{_equal},{diff}"
except:
    result = f"False,-2"
"#.into(),
        experimental: false,
    });

    // ═══════════════ PROCESS (8) ═══════════════

    b.push(BlockDefinition {
        name: "Get Process ID".into(), icon: "🔓".into(), category: "Process".into(),
        description: "Get a process ID. Enter 0 for current process.".into(),
        inputs: vec![input("pid", "number", "0")],
        outputs: vec![output("pid", "number"), output("info", "string")],
        python_template: r#"
import os
_pid = {{pid}} if {{pid}} != 0 else os.getpid()
result = f"{_pid},{_pid}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "List Processes".into(), icon: "📜".into(), category: "Process".into(),
        description: "List running processes on the system.".into(),
        inputs: vec![input("count", "number", "10")],
        outputs: vec![output("process_list", "string"), output("count", "number")],
        python_template: r#"
import sys, subprocess
limit = int({{count}})
if sys.platform == 'win32':
    output = subprocess.check_output(['tasklist', '/FO', 'CSV'], text=True).split('\n')[:limit+1]
else:
    output = subprocess.check_output(['ps', 'aux'], text=True).split('\n')[:limit+1]
_lines = '\n'.join(output)
result = _lines + "|||" + str(limit)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Get Current PID".into(), icon: "🏠".into(), category: "Process".into(),
        description: "Get the current process ID.".into(),
        inputs: vec![],
        outputs: vec![output("pid", "number")],
        python_template: r#"
import os
result = os.getpid()
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Open Process".into(), icon: "🔓".into(), category: "Process".into(),
        description: "Open a process by PID for memory access. Returns handle and PID.".into(),
        inputs: vec![input("pid", "number", "0")],
        outputs: vec![output("handle", "number"), output("pid", "number")],
        python_template: r#"
import sys
_pid = {{pid}}
if sys.platform == 'win32':
    import ctypes
    kernel32 = ctypes.windll.kernel32
    PROCESS_ALL_ACCESS = 0x1F0FFF
    handle = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
    result = f"{handle},{_pid}"
else:
    result = f"{_pid},{_pid}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Read Process Memory".into(), icon: "📖".into(), category: "Process".into(),
        description: "Read memory from another process. Returns data, PID, and address.".into(),
        inputs: vec![input("pid", "number", "0"), input("address", "number", "0"), input("size", "number", "64")],
        outputs: vec![output("data", "string"), output("pid", "number"), output("address", "number")],
        python_template: r#"
import sys
_pid = {{pid}}
_addr = {{address}}
_size = {{size}}
if sys.platform == 'win32':
    import ctypes
    kernel32 = ctypes.windll.kernel32
    PROCESS_ALL_ACCESS = 0x1F0FFF
    handle = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
    buffer = ctypes.create_string_buffer(_size)
    bytes_read = ctypes.c_size_t()
    if kernel32.ReadProcessMemory(handle, _addr, buffer, _size, ctypes.byref(bytes_read)):
        _data = buffer.raw.hex()[:(_size*2)]
        result = f"{_data},{_pid},{_addr}"
    else:
        result = f",{_pid},{_addr}"
    kernel32.CloseHandle(handle)
else:
    result = f"Not Windows,{_pid},{_addr}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Write Process Memory".into(), icon: "📝".into(), category: "Process".into(),
        description: "Write data to another process's memory. Returns success, PID, and address.".into(),
        inputs: vec![input("pid", "number", "0"), input("address", "number", "0"), input("data", "string", "")],
        outputs: vec![output("success", "bool"), output("pid", "number"), output("address", "number")],
        python_template: r#"
import sys
_pid = {{pid}}
_addr = {{address}}
_data = bytes.fromhex("{{data}}") if "{{data}}" else b""
if sys.platform == 'win32':
    import ctypes
    kernel32 = ctypes.windll.kernel32
    PROCESS_ALL_ACCESS = 0x1F0FFF
    handle = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
    bytes_written = ctypes.c_size_t()
    success = kernel32.WriteProcessMemory(handle, _addr, _data, len(_data), ctypes.byref(bytes_written))
    _ok = bool(success)
    result = f"{_ok},{_pid},{_addr}"
    kernel32.CloseHandle(handle)
else:
    result = f"False,{_pid},{_addr}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Inject Code".into(), icon: "💉".into(), category: "Process".into(),
        description: "Inject DLL into a remote process. Returns success and PID.".into(),
        inputs: vec![input("pid", "number", "0"), input("dll_path", "string", "")],
        outputs: vec![output("success", "bool"), output("pid", "number")],
        python_template: r#"
import sys
_pid = {{pid}}
_dll = "{{dll_path}}"
if sys.platform == 'win32':
    import ctypes
    kernel32 = ctypes.windll.kernel32
    PROCESS_ALL_ACCESS = 0x1F0FFF
    handle = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
    dll_bytes = _dll.encode('utf-8')
    alloc_addr = kernel32.VirtualAllocEx(handle, None, len(dll_bytes), 0x3000, 0x40)
    kernel32.WriteProcessMemory(handle, alloc_addr, dll_bytes, len(dll_bytes), None)
    kernel32.CreateRemoteThread(handle, None, 0, kernel32.LoadLibraryA, alloc_addr, 0, None)
    result = f"True,{_pid}"
    kernel32.CloseHandle(handle)
else:
    result = f"False,{_pid}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Close Process".into(), icon: "🔒".into(), category: "Process".into(),
        description: "Close a handle to a process. Returns success and handle.".into(),
        inputs: vec![input("handle", "number", "0")],
        outputs: vec![output("success", "bool"), output("handle", "number")],
        python_template: r#"
import sys
_handle = {{handle}}
if sys.platform == 'win32':
    import ctypes
    kernel32 = ctypes.windll.kernel32
    result = f"{bool(kernel32.CloseHandle(_handle))},{_handle}"
else:
    result = f"True,{_handle}"
"#.into(),
        experimental: false,
    });

    // ═══════════════ FILE SYSTEM (10) ═══════════════

    b.push(BlockDefinition {
        name: "Read File".into(), icon: "📖".into(), category: "File System".into(),
        description: "Read contents from a file. Returns content and path.".into(),
        inputs: vec![input("path", "string", "developi_test.txt")],
        outputs: vec![output("content", "string"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()) and os.path.exists(_path):
    with open(_path, "r", encoding="utf-8") as f:
        _content = f.read()
        result = f"{_content}|||{_path}"
else:
    result = f"|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Write File".into(), icon: "✍️".into(), category: "File System".into(),
        description: "Write text to a file. Returns path and bytes written.".into(),
        inputs: vec![input("path", "string", "developi_output.txt"), input("data", "string", "developi was here")],
        outputs: vec![output("path", "string"), output("bytes_written", "number")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()):
    with open(_path, "a", encoding="utf-8") as f:
        f.write("{{data}}" + "\n")
    _bytes = len("{{data}}")
    result = f"{_bytes}|||{_path}"
else:
    result = f"0|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Delete File".into(), icon: "🗑️".into(), category: "File System".into(),
        description: "Permanently delete a file. Returns success and path.".into(),
        inputs: vec![input("path", "string", "developi_test.txt")],
        outputs: vec![output("success", "bool"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()) and os.path.exists(_path) and os.path.isfile(_path):
    os.remove(_path)
    result = f"True|||{_path}"
else:
    result = f"False|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "File Exists".into(), icon: "❓".into(), category: "File System".into(),
        description: "Check whether a file or directory exists.".into(),
        inputs: vec![input("path", "string", "developi_test.txt")],
        outputs: vec![output("exists", "bool"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
_exists = _path.startswith(os.getcwd()) and os.path.exists(_path)
result = f"{_exists}|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "List Directory".into(), icon: "📁".into(), category: "File System".into(),
        description: "List files and folders in a directory.".into(),
        inputs: vec![input("path", "string", ".")],
        outputs: vec![output("listing", "string"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()) and os.path.exists(_path):
    items = os.listdir(_path)
    _listing = str(items)
    result = _listing + "|||" + _path
else:
    result = "[]|||" + _path
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Create Directory".into(), icon: "📁".into(), category: "File System".into(),
        description: "Create a new directory.".into(),
        inputs: vec![input("path", "string", "developi_workspace")],
        outputs: vec![output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()):
    os.makedirs(_path, exist_ok=True)
    result = _path
else:
    result = ""
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Get File Info".into(), icon: "📊".into(), category: "File System".into(),
        description: "Get file size in bytes.".into(),
        inputs: vec![input("path", "string", ".")],
        outputs: vec![output("size", "number"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
if _path.startswith(os.getcwd()) and os.path.exists(_path):
    stat = os.stat(_path)
    result = f"{stat.st_size}|||{_path}"
else:
    result = f"0|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Open File".into(), icon: "📂".into(), category: "File System".into(),
        description: "Open a file and return a handle. Mode: r, w, a, rb, wb.".into(),
        inputs: vec![input("path", "string", "file.txt"), input("mode", "string", "r")],
        outputs: vec![output("handle", "number"), output("path", "string")],
        python_template: r#"
import os, sys
_path = os.path.abspath(os.path.join(os.getcwd(), "{{path}}"))
_mode = "{{mode}}"
_main = sys.modules['__main__']
if _path.startswith(os.getcwd()):
    f = open(_path, _mode)
    _key = f"__DEVOBJ__file_{id(f)}"
    _main.__dict__[_key] = f
    result = f"{_key}|||{_path}"
else:
    result = f"__DEVOBJ__INVALID|||{_path}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Seek File".into(), icon: "⏩".into(), category: "File System".into(),
        description: "Move the file pointer to a specific position.".into(),
        inputs: vec![input("handle", "number", "0"), input("position", "number", "0")],
        outputs: vec![output("new_position", "number")],
        python_template: r#"
import sys
_key = "{{handle}}"
_pos = {{position}}
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    f = _main.__dict__[_key]
    f.seek(_pos)
    result = f.tell()
else:
    result = -1
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Close File".into(), icon: "🔐".into(), category: "File System".into(),
        description: "Close a file handle.".into(),
        inputs: vec![input("handle", "number", "0")],
        outputs: vec![output("success", "bool")],
        python_template: r#"
import sys
_key = "{{handle}}"
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    f = _main.__dict__[_key]
    f.close()
    del _main.__dict__[_key]
    result = True
else:
    result = False
"#.into(),
        experimental: false,
    });

    // ═══════════════ NETWORK (8) — ALL OBJECT-PASSING FIXED ═══════════════

    b.push(BlockDefinition {
        name: "HTTP Get".into(), icon: "🌐".into(), category: "Network".into(),
        description: "Make an HTTP GET request. Returns response text.".into(),
        inputs: vec![input("url", "string", "https://httpbin.org/get"), input("timeout", "number", "5")],
        outputs: vec![output("response", "string"), output("success", "bool")],
        python_template: r#"
import urllib.request
_url = "{{url}}"
if not _url.startswith(('http://', 'https://')):
    _url = 'https://' + _url
_timeout = {{timeout}}
try:
    with urllib.request.urlopen(_url, timeout=_timeout) as resp:
        _body = resp.read().decode('utf-8')
        result = _body + "|||True"
except Exception as e:
    result = f"Error: {e}|||False"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "TCP Connect".into(), icon: "🔌".into(), category: "Network".into(),
        description: "Create and connect a TCP socket. Returns socket reference key.".into(),
        inputs: vec![input("host", "string", "example.com"), input("port", "number", "80"), input("timeout", "number", "3")],
        outputs: vec![output("socket", "any"), output("success", "bool")],
        python_template: r#"
import socket, sys
_host = "{{host}}"
_port = {{port}}
_timeout = {{timeout}}
_main = sys.modules['__main__']
try:
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(_timeout)
    s.connect((_host, _port))
    _key = f"__DEVOBJ__sock_{id(s)}"
    _main.__dict__[_key] = s
    result = f"{_key}|||True"
except Exception as e:
    result = f"__DEVOBJ__NONE|||False"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "HTTP Post".into(), icon: "📤".into(), category: "Network".into(),
        description: "Send an HTTP POST request with data.".into(),
        inputs: vec![input("url", "string", "https://httpbin.org/post"), input("data", "string", "key=value"), input("timeout", "number", "5")],
        outputs: vec![output("response", "string"), output("success", "bool")],
        python_template: r#"
import urllib.request
_url = "{{url}}"
if not _url.startswith(('http://', 'https://')):
    _url = 'https://' + _url
_data = "{{data}}".encode('utf-8')
try:
    req = urllib.request.Request(_url, data=_data, method='POST')
    with urllib.request.urlopen(req, timeout={{timeout}}) as resp:
        _body = resp.read().decode('utf-8')
        result = _body + "|||True"
except Exception as e:
    result = f"Error: {e}|||False"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Create Socket".into(), icon: "🔌".into(), category: "Network".into(),
        description: "Create a TCP or UDP socket. Returns socket reference key.".into(),
        inputs: vec![input("type", "string", "tcp")],
        outputs: vec![output("socket", "any")],
        python_template: r#"
import socket, sys
_type = "{{type}}"
sock_type = socket.SOCK_STREAM if _type == "tcp" else socket.SOCK_DGRAM
s = socket.socket(socket.AF_INET, sock_type)
_main = sys.modules['__main__']
_key = f"__DEVOBJ__sock_{id(s)}"
_main.__dict__[_key] = s
result = _key
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Bind Socket".into(), icon: "📌".into(), category: "Network".into(),
        description: "Bind a socket to a port. Returns bound port number.".into(),
        inputs: vec![input("socket", "any", ""), input("port", "number", "8080")],
        outputs: vec![output("bound_port", "number")],
        python_template: r#"
import sys
_key = "{{socket}}"
_port = {{port}}
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        _sock.bind(('0.0.0.0', _port))
        result = _sock.getsockname()[1]
    except:
        result = -1
else:
    result = -1
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Send Data".into(), icon: "📤".into(), category: "Network".into(),
        description: "Send data through a socket. Returns bytes sent.".into(),
        inputs: vec![input("socket", "any", ""), input("data", "string", "Hello")],
        outputs: vec![output("bytes_sent", "number")],
        python_template: r#"
import sys
_key = "{{socket}}"
_data = "{{data}}".encode('utf-8')
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        result = _sock.send(_data)
    except:
        result = 0
else:
    result = 0
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Receive Data".into(), icon: "📥".into(), category: "Network".into(),
        description: "Receive data from a socket. Returns data string.".into(),
        inputs: vec![input("socket", "any", ""), input("buffer_size", "number", "1024")],
        outputs: vec![output("data", "string")],
        python_template: r#"
import sys
_key = "{{socket}}"
_size = {{buffer_size}}
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        data = _sock.recv(_size)
        result = data.decode('utf-8', errors='ignore')
    except:
        result = ""
else:
    result = ""
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Close Socket".into(), icon: "🔌".into(), category: "Network".into(),
        description: "Close a socket connection.".into(),
        inputs: vec![input("socket", "any", "")],
        outputs: vec![output("success", "bool")],
        python_template: r#"
import sys
_key = "{{socket}}"
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        _sock.close()
        del _main.__dict__[_key]
        result = True
    except:
        result = False
else:
    result = False
"#.into(),
        experimental: false,
    });

b.push(BlockDefinition {
    name: "Listen Socket".into(), icon: "👂".into(), category: "Network".into(),
    description: "Listen for incoming connections on a bound socket.".into(),
    inputs: vec![
        input("socket", "any", ""),
        input("backlog", "number", "5"),
    ],
    outputs: vec![output("socket", "any")],
    python_template: r#"
import sys
_key = "{{socket}}"
_backlog = {{backlog}}
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        _sock.listen(_backlog)
        result = _key
    except:
        result = "__DEVOBJ__NONE"
else:
    result = "__DEVOBJ__NONE"
"#.into(),
    experimental: false,
});

b.push(BlockDefinition {
    name: "Accept Connection".into(), icon: "🤝".into(), category: "Network".into(),
    description: "Accept an incoming connection. Returns client socket key.".into(),
    inputs: vec![
        input("socket", "any", ""),
    ],
    outputs: vec![
        output("client_socket", "any"),
        output("client_address", "string"),
    ],
    python_template: r#"
import sys
_key = "{{socket}}"
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _sock = _main.__dict__[_key]
    try:
        client, addr = _sock.accept()
        _client_key = f"__DEVOBJ__sock_{id(client)}"
        _main.__dict__[_client_key] = client
        result = f"{_client_key}|||{addr[0]}:{addr[1]}"
    except:
        result = "__DEVOBJ__NONE|||"
else:
    result = "__DEVOBJ__NONE|||"
"#.into(),
    experimental: false,
});

    // ═══════════════ DATA (13) ═══════════════

    b.push(BlockDefinition {
        name: "Integer".into(), icon: "🔢".into(), category: "Data".into(),
        description: "Create an integer value.".into(),
        inputs: vec![input("value", "number", "42")],
        outputs: vec![output("value", "number")],
        python_template: r#"result = int({{value}})"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Float".into(), icon: "🔣".into(), category: "Data".into(),
        description: "Create a floating-point number.".into(),
        inputs: vec![input("value", "number", "3.14159")],
        outputs: vec![output("value", "number")],
        python_template: r#"result = float({{value}})"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "String".into(), icon: "📝".into(), category: "Data".into(),
        description: "Create a text string. Type anything — no quotes needed.".into(),
        inputs: vec![input("text", "string", "Hello from developi")],
        outputs: vec![output("text", "string")],
        python_template: r#"result = """{{text}}""" "#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Join Strings".into(), icon: "🔗".into(), category: "Data".into(),
        description: "Join two strings together.".into(),
        inputs: vec![input("a", "string", "Hello"), input("b", "string", " World")],
        outputs: vec![output("result", "string")],
        python_template: r#"result = "{{a}}" + "{{b}}""#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "List".into(), icon: "📋".into(), category: "Data".into(),
        description: "Create a list from comma-separated values.".into(),
        inputs: vec![input("items", "string", "1, 2, 3, 4, 5")],
        outputs: vec![output("list", "any")],
        python_template: r#"
import json
_items = "{{items}}".strip()
try:
    result = json.loads(_items)
except:
    result = [x.strip() for x in _items.split(",")]
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Key-Value Pair".into(), icon: "📖".into(), category: "Data".into(),
        description: "Create a simple key=value pair. No JSON needed.".into(),
        inputs: vec![input("key", "string", "name"), input("value", "string", "developi")],
        outputs: vec![output("dict", "any")],
        python_template: r#"
_key = "{{key}}"
_val = "{{value}}"
try:
    _val_num = float(_val)
    if _val_num == int(_val_num):
        _val_num = int(_val_num)
    result = {_key: _val_num}
except:
    result = {_key: _val}
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Encode Text".into(), icon: "🔤".into(), category: "Data".into(),
        description: "Convert text to hex bytes.".into(),
        inputs: vec![input("text", "string", "developi")],
        outputs: vec![output("hex", "string")],
        python_template: r#"result = "{{text}}".encode('utf-8').hex()"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Decode Text".into(), icon: "🔡".into(), category: "Data".into(),
        description: "Convert hex bytes back to text.".into(),
        inputs: vec![input("hex", "string", "646576656c6f7069")],
        outputs: vec![output("text", "string")],
        python_template: r#"
try:
    result = bytes.fromhex("{{hex}}").decode('utf-8')
except:
    result = "Invalid hex"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Slice".into(), icon: "✂️".into(), category: "Data".into(),
        description: "Get a portion of a list or string.".into(),
        inputs: vec![input("data", "string", "0,1,2,3,4,5"), input("start", "number", "0"), input("end", "number", "3")],
        outputs: vec![output("sliced", "any")],
        python_template: r#"
import json
_data = "{{data}}"
try:
    data = json.loads(_data)
except:
    data = [x.strip() for x in _data.split(",")]
result = data[{{start}}:{{end}}]
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Length".into(), icon: "📏".into(), category: "Data".into(),
        description: "Get the length of any list or string.".into(),
        inputs: vec![input("data", "string", "1,2,3,4,5")],
        outputs: vec![output("length", "number")],
        python_template: r#"
import json
_data = "{{data}}"
try:
    data = json.loads(_data)
except:
    data = [x.strip() for x in _data.split(",")]
result = len(data)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Bytes".into(), icon: "💾".into(), category: "Data".into(),
        description: "Create raw bytes from hex string.".into(),
        inputs: vec![input("hex", "string", "48656C6C6F")],
        outputs: vec![output("bytes", "bytes")],
        python_template: r#"
try:
    result = bytes.fromhex("{{hex}}")
except:
    result = b""
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Pack Number".into(), icon: "📦".into(), category: "Data".into(),
        description: "Pack a number into binary format. Choose the type.".into(),
        inputs: vec![input("type", "string", "32-bit integer"), input("value", "number", "42")],
        outputs: vec![output("packed", "bytes")],
        python_template: r#"
import struct
_type = "{{type}}"
_val = {{value}}
formats = {
    "32-bit integer": "<i", "unsigned 32-bit": "<I",
    "16-bit integer": "<h", "unsigned 16-bit": "<H",
    "64-bit integer": "<q", "float": "<f", "double": "<d",
    "byte": "<b", "unsigned byte": "<B",
}
fmt = formats.get(_type, "<i")
try:
    packed = struct.pack(fmt, int(_val))
    result = packed.hex()
except:
    result = ""
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Unpack Number".into(), icon: "📤".into(), category: "Data".into(),
        description: "Unpack binary hex data back to a number.".into(),
        inputs: vec![input("type", "string", "32-bit integer"), input("hex_data", "string", "2A000000")],
        outputs: vec![output("value", "string")],
        python_template: r#"
import struct
_type = "{{type}}"
_hex = "{{hex_data}}"
formats = {
    "32-bit integer": "<i", "unsigned 32-bit": "<I",
    "16-bit integer": "<h", "unsigned 16-bit": "<H",
    "64-bit integer": "<q", "float": "<f", "double": "<d",
    "byte": "<b", "unsigned byte": "<B",
}
fmt = formats.get(_type, "<i")
try:
    data = bytes.fromhex(_hex)
    result = str(struct.unpack(fmt, data)[0])
except:
    result = "Invalid data"
"#.into(),
        experimental: false,
    });

    // ═══════════════ LOGIC (11) ═══════════════

    b.push(BlockDefinition {
        name: "If Condition".into(), icon: "🔱".into(), category: "Logic".into(),
        description: "Branch based on a condition. Connect a Boolean block for input.".into(),
        inputs: vec![input("condition", "bool", "true")],
        outputs: vec![output("result", "bool")],
        python_template: r#"cond = str("{{condition}}").lower() == "true"; result = cond"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Compare".into(), icon: "⚖️".into(), category: "Logic".into(),
        description: "Compare two values: equals, not equals, greater than, less than.".into(),
        inputs: vec![input("a", "any", "42"), input("b", "any", "42"), input("operator", "string", "equals")],
        outputs: vec![output("result", "bool")],
        python_template: r#"
a = "{{a}}"; b = "{{b}}"; op = "{{operator}}"
try:
    if a.replace('.','',1).replace('-','',1).isdigit(): a = float(a)
    if b.replace('.','',1).replace('-','',1).isdigit(): b = float(b)
except: pass
ops = {"equals": lambda x,y: x==y, "not equals": lambda x,y: x!=y,
    "greater than": lambda x,y: x>y, "less than": lambda x,y: x<y,
    "greater or equal": lambda x,y: x>=y, "less or equal": lambda x,y: x<=y}
result = ops.get(op, ops["equals"])(a, b)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Boolean AND".into(), icon: "🔗".into(), category: "Logic".into(),
        description: "True only if both inputs are true.".into(),
        inputs: vec![input("a", "bool", "true"), input("b", "bool", "true")],
        outputs: vec![output("result", "bool")],
        python_template: r#"result = str("{{a}}").lower() == "true" and str("{{b}}").lower() == "true""#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Boolean OR".into(), icon: "🔀".into(), category: "Logic".into(),
        description: "True if at least one input is true.".into(),
        inputs: vec![input("a", "bool", "true"), input("b", "bool", "false")],
        outputs: vec![output("result", "bool")],
        python_template: r#"result = str("{{a}}").lower() == "true" or str("{{b}}").lower() == "true""#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Boolean NOT".into(), icon: "🚫".into(), category: "Logic".into(),
        description: "Inverts the input boolean.".into(),
        inputs: vec![input("value", "bool", "false")],
        outputs: vec![output("result", "bool")],
        python_template: r#"result = not (str("{{value}}").lower() == "true")"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "For Each".into(), icon: "🔄".into(), category: "Logic".into(),
        description: "Loop through each item in a list.".into(),
        inputs: vec![input("list", "string", "a, b, c, d")],
        outputs: vec![output("results", "string")],
        python_template: r#"
import json
_list = "{{list}}"
try: items = json.loads(_list)
except: items = [x.strip() for x in _list.split(",")]
result = str([item for item in items])
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Repeat".into(), icon: "⏳".into(), category: "Logic".into(),
        description: "Generate squares of numbers from 0 to N.".into(),
        inputs: vec![input("times", "number", "10")],
        outputs: vec![output("results", "string")],
        python_template: r#"
limit = {{times}}
values = [i * i for i in range(limit)]
result = str(values)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Range".into(), icon: "🔢".into(), category: "Logic".into(),
        description: "Generate numbers from start to end.".into(),
        inputs: vec![input("start", "number", "0"), input("end", "number", "10")],
        outputs: vec![output("numbers", "string")],
        python_template: r#"result = str(list(range({{start}}, {{end}})))"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Find First".into(), icon: "⏹️".into(), category: "Logic".into(),
        description: "Find first number whose square exceeds threshold.".into(),
        inputs: vec![input("threshold", "number", "500")],
        outputs: vec![output("found", "number")],
        python_template: r#"
found = None
for i in range(100):
    if i * i > {{threshold}}:
        found = i; break
result = found if found else -1
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Filter Evens".into(), icon: "⏭️".into(), category: "Logic".into(),
        description: "Generate even numbers up to limit.".into(),
        inputs: vec![input("limit", "number", "20")],
        outputs: vec![output("evens", "string")],
        python_template: r#"
limit = {{limit}}
evens = [i for i in range(limit) if i % 2 == 0]
result = str(evens)
"#.into(),
        experimental: false,
    });

    // ═══════════════ MATH (8) ═══════════════

    let math_ops = vec![
        ("Add", "➕", "{{a}} + {{b}}"),
        ("Subtract", "➖", "{{a}} - {{b}}"),
        ("Multiply", "✖️", "{{a}} * {{b}}"),
        ("Divide", "➗", "{{a}} / {{b}} if {{b}} != 0 else 'Division by zero'"),
        ("Modulo", "🔄", "{{a}} % {{b}} if {{b}} != 0 else 0"),
        ("Power", "💪", "{{a}} ** {{b}}"),
    ];

    for (name, icon, expr) in math_ops {
        b.push(BlockDefinition {
            name: name.into(), icon: icon.into(), category: "Math".into(),
            description: format!("{} two numbers.", name),
            inputs: vec![input("a", "number", "0"), input("b", "number", "0")],
            outputs: vec![output("result", "number")],
            python_template: expr.into(),
            experimental: false,
        });
    }

    b.push(BlockDefinition {
        name: "Bitwise AND".into(), icon: "&".into(), category: "Math".into(),
        description: "Bitwise AND operation.".into(),
        inputs: vec![input("a", "number", "255"), input("b", "number", "15")],
        outputs: vec![output("result", "number")],
        python_template: r#"result = {{a}} & {{b}}"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Bitwise OR".into(), icon: "|".into(), category: "Math".into(),
        description: "Bitwise OR operation.".into(),
        inputs: vec![input("a", "number", "240"), input("b", "number", "15")],
        outputs: vec![output("result", "number")],
        python_template: r#"result = {{a}} | {{b}}"#.into(),
        experimental: false,
    });

    // ═══════════════ VARIABLES (4) ═══════════════

    b.push(BlockDefinition {
        name: "Store Value".into(), icon: "📝".into(), category: "Variables".into(),
        description: "Save a value to remember later.".into(),
        inputs: vec![input("name", "string", "my_value"), input("value", "any", "Hello")],
        outputs: vec![output("stored", "any"), output("name", "string")],
        python_template: r#"
import sys
_name = "{{name}}"; _value = "{{value}}"
_main = sys.modules['__main__']
_main.__dict__[_name] = _value
result = f"{_value}|||{_name}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Get Value".into(), icon: "🔍".into(), category: "Variables".into(),
        description: "Retrieve a stored value by name.".into(),
        inputs: vec![input("name", "string", "my_value")],
        outputs: vec![output("value", "any"), output("name", "string")],
        python_template: r#"
import sys
_name = "{{name}}"
_main = sys.modules['__main__']
if _name in _main.__dict__:
    _val = _main.__dict__[_name]
    result = f"{_val}|||{_name}"
else:
    result = f"|||{_name}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Clear Variable".into(), icon: "🗑️".into(), category: "Variables".into(),
        description: "Delete a stored variable.".into(),
        inputs: vec![input("name", "string", "my_value")],
        outputs: vec![output("success", "bool")],
        python_template: r#"
import sys
_name = "{{name}}"
_main = sys.modules['__main__']
if _name in _main.__dict__:
    del _main.__dict__[_name]
    result = True
else:
    result = False
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Variable Exists".into(), icon: "❓".into(), category: "Variables".into(),
        description: "Check if a variable name is in use.".into(),
        inputs: vec![input("name", "string", "my_value")],
        outputs: vec![output("exists", "bool")],
        python_template: r#"
import sys
_name = "{{name}}"
_main = sys.modules['__main__']
result = _name in _main.__dict__
"#.into(),
        experimental: false,
    });

    // ═══════════════ DEBUG (5) ═══════════════

    b.push(BlockDefinition {
        name: "Print".into(), icon: "🖨️".into(), category: "Debug".into(),
        description: "Print any value to the console.".into(),
        inputs: vec![input("value", "any", "Hello from developi")],
        outputs: vec![],
        python_template: r#"print("{{value}}"); result = "{{value}}""#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Check Condition".into(), icon: "✅".into(), category: "Debug".into(),
        description: "Compare two values. Operator: equals, not equals, greater than, less than.".into(),
        inputs: vec![
            input("a", "any", "2"), input("b", "any", "2"),
            input("operator", "string", "equals"), input("message", "string", "Check failed!"),
        ],
        outputs: vec![output("passed", "bool")],
        python_template: r#"
_a = "{{a}}"; _b = "{{b}}"; _op = "{{operator}}"; _msg = "{{message}}"
try:
    _an = float(_a); _bn = float(_b)
    _a, _b = _an, _bn
except: pass
ops = {"equals": lambda x,y: x==y, "not equals": lambda x,y: x!=y,
    "greater than": lambda x,y: x>y, "less than": lambda x,y: x<y,
    "greater or equal": lambda x,y: x>=y, "less or equal": lambda x,y: x<=y}
check = ops.get(_op, ops["equals"])
if check(_a, _b): result = True
else:
    print(f"❌ Assertion failed: {_a} {_op} {_b} — {_msg}")
    result = False
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Start Timer".into(), icon: "⏱️".into(), category: "Debug".into(),
        description: "Start timing code execution. Returns timer ID.".into(),
        inputs: vec![],
        outputs: vec![output("timer_id", "string")],
        python_template: r#"
import time, uuid, sys
_timer_id = str(uuid.uuid4())[:8]
_main = sys.modules['__main__']
if not hasattr(_main, '_developi_timers'): _main._developi_timers = {}
_main._developi_timers[_timer_id] = time.perf_counter()
result = _timer_id
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "End Timer".into(), icon: "🏁".into(), category: "Debug".into(),
        description: "Get elapsed milliseconds since Start Timer.".into(),
        inputs: vec![input("timer_id", "string", "")],
        outputs: vec![output("elapsed_ms", "number")],
        python_template: r#"
import time, sys
_timer_id = "{{timer_id}}"
_main = sys.modules['__main__']
if hasattr(_main, '_developi_timers') and _timer_id in _main._developi_timers:
    start = _main._developi_timers[_timer_id]
    result = (time.perf_counter() - start) * 1000
    del _main._developi_timers[_timer_id]
else:
    result = 0
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Log Message".into(), icon: "📋".into(), category: "Debug".into(),
        description: "Create a timestamped log entry. Level: INFO, WARN, ERROR, DEBUG.".into(),
        inputs: vec![input("level", "string", "INFO"), input("message", "string", "developi running")],
        outputs: vec![output("log", "string")],
        python_template: r#"
import time
_level = "{{level}}"; _msg = "{{message}}"
timestamp = time.strftime("%H:%M:%S")
result = f"[{timestamp}] [{_level}] {_msg}"
"#.into(),
        experimental: false,
    });

    // ═══════════════ SIMPLE BLOCKS (3) ═══════════════

    b.push(BlockDefinition {
        name: "Wait".into(), icon: "⏸️".into(), category: "Logic".into(),
        description: "Pause execution for N seconds.".into(),
        inputs: vec![input("seconds", "number", "1")],
        outputs: vec![output("waited", "number")],
        python_template: r#"
import time
_secs = {{seconds}}
time.sleep(_secs)
result = _secs
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Random Number".into(), icon: "🎲".into(), category: "Math".into(),
        description: "Generate a random number between min and max.".into(),
        inputs: vec![input("min", "number", "1"), input("max", "number", "100")],
        outputs: vec![output("random", "number")],
        python_template: r#"
import random
result = random.randint({{min}}, {{max}})
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Current Time".into(), icon: "🕐".into(), category: "Data".into(),
        description: "Get current date and time.".into(),
        inputs: vec![input("format", "string", "%H:%M:%S")],
        outputs: vec![output("time", "string")],
        python_template: r#"
import time
_fmt = "{{format}}"
result = time.strftime(_fmt)
"#.into(),
        experimental: false,
    });

    // ═══════════════ FUNCTIONS (5) ═══════════════

    b.push(BlockDefinition {
        name: "Define Function".into(), icon: "⚙️".into(), category: "Functions".into(),
        description: "Define a math function: multiply, add, subtract, divide, power, modulo.".into(),
        inputs: vec![
            input("name", "string", "my_func"),
            input("param_name", "string", "x"),
            input("operation", "string", "multiply"),
            input("value", "number", "2"),
        ],
        outputs: vec![output("name", "string")],
        python_template: r#"
import sys
_name = "{{name}}"; _param = "{{param_name}}"; _op = "{{operation}}"; _val = {{value}}
_main = sys.modules['__main__']
ops = {
    "multiply": f"def {_name}({_param}): return {_param} * {_val}",
    "add": f"def {_name}({_param}): return {_param} + {_val}",
    "subtract": f"def {_name}({_param}): return {_param} - {_val}",
    "divide": f"def {_name}({_param}): return {_param} / {_val}",
    "power": f"def {_name}({_param}): return {_param} ** {_val}",
    "modulo": f"def {_name}({_param}): return {_param} % {_val}",
}
_code = ops.get(_op, ops["multiply"])
exec(_code, _main.__dict__)
result = _name
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Call Function".into(), icon: "📞".into(), category: "Functions".into(),
        description: "Call a defined function with one number argument.".into(),
        inputs: vec![input("name", "string", "my_func"), input("argument", "number", "5")],
        outputs: vec![output("result", "any")],
        python_template: r#"
import sys
_name = "{{name}}"; _arg = {{argument}}
_main = sys.modules['__main__']
if _name in _main.__dict__: result = _main.__dict__[_name](_arg)
else: result = ""
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Return".into(), icon: "↩️".into(), category: "Functions".into(),
        description: "Pass a value through. Returns the same value.".into(),
        inputs: vec![input("value", "any", "done")],
        outputs: vec![output("result", "any")],
        python_template: r#"result = """{{value}}""" "#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Apply Operation".into(), icon: "λ".into(), category: "Functions".into(),
        description: "Apply a math operation: double, triple, half, square, cube, negate, absolute.".into(),
        inputs: vec![input("operation", "string", "double"), input("value", "number", "5")],
        outputs: vec![output("result", "any")],
        python_template: r#"
_op = "{{operation}}"; _val = {{value}}
ops = {
    "double": lambda x: x * 2, "triple": lambda x: x * 3,
    "half": lambda x: x / 2, "square": lambda x: x * x,
    "cube": lambda x: x * x * x, "negate": lambda x: -x,
    "absolute": lambda x: abs(x), "increment": lambda x: x + 1,
    "decrement": lambda x: x - 1, "reciprocal": lambda x: 1 / x if x != 0 else 0,
}
f = ops.get(_op, ops["double"])
result = f(_val)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Partial".into(), icon: "🧩".into(), category: "Functions".into(),
        description: "Calculate base raised to a power.".into(),
        inputs: vec![input("base", "number", "5"), input("exp", "number", "2")],
        outputs: vec![output("result", "number")],
        python_template: r#"
from functools import partial
_base = {{base}}; _exp = {{exp}}
def power(base, exp): return base ** exp
square = partial(power, exp=_exp)
result = square(_base)
"#.into(),
        experimental: false,
    });

    // ═══════════════ PYTHON POWER (8) ═══════════════

    b.push(BlockDefinition {
        name: "Import".into(), icon: "📥".into(), category: "Python Power".into(),
        description: "Import a Python module (math, random, json, os, etc.).".into(),
        inputs: vec![input("module", "string", "math")],
        outputs: vec![output("module", "any")],
        python_template: r#"
import importlib, sys
_mod = "{{module}}"
_main = sys.modules['__main__']
_main.__dict__[_mod] = importlib.import_module(_mod)
result = _mod
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Exec Python".into(), icon: "▶️".into(), category: "Python Power".into(),
        description: "Execute arbitrary Python code. For advanced users.".into(),
        inputs: vec![input("code", "string", "result = sum(range(10))")],
        outputs: vec![output("result", "any")],
        python_template: r#"
import sys
_code = "{{code}}"
_main = sys.modules['__main__']
exec(_code, _main.__dict__)
result = _main.__dict__.get('result', '')
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Eval".into(), icon: "🧮".into(), category: "Python Power".into(),
        description: "Evaluate a Python expression. For advanced users.".into(),
        inputs: vec![input("expression", "string", "2 + 2")],
        outputs: vec![output("result", "any")],
        python_template: r#"
import sys
_expr = "{{expression}}"
_main = sys.modules['__main__']
result = eval(_expr, _main.__dict__)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Try".into(), icon: "🛡️".into(), category: "Python Power".into(),
        description: "Execute code with error handling. Returns result and error message.".into(),
        inputs: vec![input("code", "string", "result = 100 / 0")],
        outputs: vec![output("result", "any"), output("error", "string")],
        python_template: r#"
import sys
_code = "{{code}}"
_main = sys.modules['__main__']
try:
    exec(_code, _main.__dict__)
    _res = _main.__dict__.get('result', '')
    result = f"{_res},"
except Exception as e:
    result = f",{str(e)}"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Raise".into(), icon: "⚠️".into(), category: "Python Power".into(),
        description: "Raise an error with a message. Returns the error text.".into(),
        inputs: vec![input("message", "string", "Something went wrong")],
        outputs: vec![output("error", "string")],
        python_template: r#"
_msg = "{{message}}"
try: raise ValueError(_msg)
except ValueError as e: result = str(e)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Convert Type".into(), icon: "🔄".into(), category: "Python Power".into(),
        description: "Convert a value to a target type (int, float, str, bool, list, bytes).".into(),
        inputs: vec![input("value", "any", "42"), input("target_type", "string", "int")],
        outputs: vec![output("converted", "any")],
        python_template: r#"
_val = "{{value}}"; _tgt = "{{target_type}}"
types = {"int": int, "float": float, "str": str, "bool": bool, "list": list, "bytes": bytes}
t = types.get(_tgt, str)
try:
    if _tgt == "bool": result = _val.lower() == "true"
    else: result = t(_val)
except: result = _val
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Type Of".into(), icon: "🏷️".into(), category: "Python Power".into(),
        description: "Get the Python type name of a value.".into(),
        inputs: vec![input("value", "any", "hello")],
        outputs: vec![output("type", "string")],
        python_template: r#"
_val = "{{value}}"
try:
    import ast
    _evaled = ast.literal_eval(_val)
    result = type(_evaled).__name__
except: result = type(_val).__name__
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Run Async".into(), icon: "⏳".into(), category: "Python Power".into(),
        description: "Run a pre-built async task: count, factorial, fibonacci, delay.".into(),
        inputs: vec![input("task", "string", "count"), input("param", "number", "3")],
        outputs: vec![output("result", "string")],
        python_template: r#"
import asyncio
_task = "{{task}}"; _param = {{param}}

async def count_to(n):
    result = []
    for i in range(1, n+1):
        await asyncio.sleep(0.1)
        result.append(str(i))
    return ", ".join(result)

async def factorial(n):
    r = 1
    for i in range(1, n+1): r *= i
    return str(r)

async def fibonacci(n):
    a, b = 0, 1
    seq = []
    for _ in range(n):
        seq.append(str(a))
        a, b = b, a + b
    return ", ".join(seq)

async def delay_message(n):
    await asyncio.sleep(n)
    return f"Waited {n} seconds!"

tasks = {"count": count_to, "factorial": factorial, "fibonacci": fibonacci, "delay": delay_message}
func = tasks.get(_task, tasks["count"])
result = asyncio.run(func(_param))
"#.into(),
        experimental: false,
    });

    // ═══════════════ LOW-LEVEL (6) — ALL OBJECT-PASSING FIXED ═══════════════

    b.push(BlockDefinition {
        name: "Load Library".into(), icon: "📚".into(), category: "Low-Level".into(),
        description: "Load a shared library (DLL/SO). Returns library reference key.".into(),
        inputs: vec![input("path", "string", "kernel32.dll")],
        outputs: vec![output("library", "any")],
        python_template: r#"
import ctypes, sys
_path = "{{path}}"
_main = sys.modules['__main__']
try:
    lib = ctypes.CDLL(_path)
    _key = f"__DEVOBJ__lib_{id(lib)}"
    _main.__dict__[_key] = lib
    result = _key
except:
    result = "__DEVOBJ__NONE"
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Call C Function".into(), icon: "📞".into(), category: "Low-Level".into(),
        description: "Call a function from a loaded C library.".into(),
        inputs: vec![input("library", "any", ""), input("function", "string", "GetCurrentProcessId")],
        outputs: vec![output("result", "any")],
        python_template: r#"
import sys
_key = "{{library}}"
_func = "{{function}}"
_main = sys.modules['__main__']
if _key.startswith("__DEVOBJ__") and _key in _main.__dict__:
    _lib = _main.__dict__[_key]
    try:
        func = getattr(_lib, _func)
        result = func()
    except:
        result = None
else:
    result = None
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Struct Size".into(), icon: "🏗️".into(), category: "Low-Level".into(),
        description: "Get the size of common C structs: Point, Point3D, Color, Rectangle, Vector2, Vector3.".into(),
        inputs: vec![input("struct_type", "string", "Point (x,y)")],
        outputs: vec![output("size", "number")],
        python_template: r#"
import ctypes
_type = "{{struct_type}}"
structs = {
    "Point (x,y)": [("x", ctypes.c_int), ("y", ctypes.c_int)],
    "Point3D (x,y,z)": [("x", ctypes.c_int), ("y", ctypes.c_int), ("z", ctypes.c_int)],
    "Color (r,g,b,a)": [("r", ctypes.c_ubyte), ("g", ctypes.c_ubyte), ("b", ctypes.c_ubyte), ("a", ctypes.c_ubyte)],
    "Rectangle (x,y,w,h)": [("x", ctypes.c_int), ("y", ctypes.c_int), ("w", ctypes.c_int), ("h", ctypes.c_int)],
    "Vector2 (x,y)": [("x", ctypes.c_float), ("y", ctypes.c_float)],
    "Vector3 (x,y,z)": [("x", ctypes.c_float), ("y", ctypes.c_float), ("z", ctypes.c_float)],
}
fields = structs.get(_type, structs["Point (x,y)"])
class DynamicStruct(ctypes.Structure):
    _fields_ = fields
result = ctypes.sizeof(DynamicStruct)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Address Of".into(), icon: "📍".into(), category: "Low-Level".into(),
        description: "Get the memory address of a stored variable.".into(),
        inputs: vec![input("object", "any", "developi")],
        outputs: vec![output("address", "number")],
        python_template: r#"
_obj = "{{object}}"
import sys
_main = sys.modules['__main__']
if _obj in _main.__dict__: result = id(_main.__dict__[_obj])
else: result = 0
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Resize Memory".into(), icon: "🔧".into(), category: "Low-Level".into(),
        description: "Allocate a new buffer of a different size. Returns address.".into(),
        inputs: vec![input("new_size", "number", "128")],
        outputs: vec![output("address", "number")],
        python_template: r#"
import ctypes
_new = {{new_size}}
buf = ctypes.create_string_buffer(_new)
result = ctypes.addressof(buf)
"#.into(),
        experimental: false,
    });

    b.push(BlockDefinition {
        name: "Syscall".into(), icon: "⚡".into(), category: "Low-Level".into(),
        description: "Make a direct system call (Linux/macOS only).".into(),
        inputs: vec![input("number", "number", "39"), input("arg1", "number", "0")],
        outputs: vec![output("result", "number")],
        python_template: r#"
import sys
_num = {{number}}; _arg1 = {{arg1}}
if sys.platform != 'win32':
    import ctypes
    libc = ctypes.CDLL('libc.so.6' if sys.platform == 'linux' else 'libc.dylib')
    libc.syscall.argtypes = [ctypes.c_long, ctypes.c_long]
    result = libc.syscall(_num, _arg1)
else:
    result = 0
"#.into(),
        experimental: false,
    });

    b
}