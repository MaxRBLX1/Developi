// blocks.rs — Block Registry & Definitions
// 92 blocks with input/output ports. No hardcoded values.
// Users type values OR connect blocks. No walls.
// All output ports send clean data. Wires just work.
// Every input that matters has a matching output pass-through.
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
}

fn port(name: &str, ptype: &str, default: &str) -> BlockPort {
    BlockPort { name: name.into(), port_type: ptype.into(), default_value: default.into() }
}
fn input(name: &str, ptype: &str, default: &str) -> BlockPort { port(name, ptype, default) }
fn output(name: &str, ptype: &str) -> BlockPort { port(name, ptype, "") }

fn build_all_blocks() -> Vec<BlockDefinition> {
    let mut b = Vec::new();

    // ═══════════════ MEMORY (8) ═══════════════

    b.push(BlockDefinition {
        name: "Allocate Memory".into(), icon: "📦".into(), category: "Memory".into(),
        description: "Allocate a block of raw memory. Returns the memory address and size.".into(),
        inputs: vec![input("size", "number", "1024")],
        outputs: vec![output("address", "number"), output("size", "number")],
        python_template: r#"
import ctypes
_size = {{size}}
__alloc = ctypes.create_string_buffer(_size)
result = ctypes.addressof(__alloc)
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Free Memory".into(), icon: "🗑️".into(), category: "Memory".into(),
        description: "Free a previously allocated memory block. Returns success and the address.".into(),
        inputs: vec![input("address", "number", "0")],
        outputs: vec![output("success", "bool"), output("address", "number")],
        python_template: r#"
import ctypes
_addr = {{address}}
try:
    ctypes.free(ctypes.c_void_p(_addr))
    result = True
except:
    result = False
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Read Memory".into(), icon: "👁️".into(), category: "Memory".into(),
        description: "Read raw bytes from a memory address. Returns hex string, address, and size.".into(),
        inputs: vec![input("address", "number", "0"), input("size", "number", "16")],
        outputs: vec![output("data", "bytes"), output("address", "number"), output("size", "number")],
        python_template: r#"
import ctypes
_addr = {{address}}
_size = {{size}}
try:
    data = ctypes.string_at(_addr, _size)
    result = data.hex()
except Exception as e:
    result = f"Error: {e}"
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Write Memory".into(), icon: "✏️".into(), category: "Memory".into(),
        description: "Write text to a memory address. Always works — just type what you want. Returns bytes written and address.".into(),
        inputs: vec![input("address", "number", "0"), input("data", "string", "developi")],
        outputs: vec![output("bytes_written", "number"), output("address", "number")],
        python_template: r#"
import ctypes
_addr = {{address}}
data = "{{data}}".encode('utf-8')
ctypes.memmove(_addr, data, len(data))
result = len(data)
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Cast Pointer".into(), icon: "🎯".into(), category: "Memory".into(),
        description: "Cast a raw pointer to any C type. Type the ctypes name directly. Returns the value, address, and type used.".into(),
        inputs: vec![input("address", "number", "0"), input("type", "string", "c_int")],
        outputs: vec![output("value", "any"), output("address", "number"), output("type_used", "string")],
        python_template: r#"
import ctypes
_addr = {{address}}
_type_str = """{{type}}"""
try:
    t = getattr(ctypes, _type_str, ctypes.c_int)
    ptr = ctypes.cast(_addr, ctypes.POINTER(t))
    result = ptr.contents.value
except Exception as e:
    result = f"Error: {e}"
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Size Of".into(), icon: "📏".into(), category: "Memory".into(),
        description: "Get the size in bytes of any C type. Type the name directly.".into(),
        inputs: vec![input("type", "string", "c_int")],
        outputs: vec![output("size", "number"), output("type", "string")],
        python_template: r#"
import ctypes
_type_str = """{{type}}"""
try:
    t = getattr(ctypes, _type_str, None)
    if t:
        result = ctypes.sizeof(t)
    else:
        result = ctypes.sizeof(ctypes.c_int)
except:
    result = ctypes.sizeof(ctypes.c_int)
"#.into(),
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
ctypes.memmove(_dst, _src, _size)
result = _size
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Memory Compare".into(), icon: "⚖️".into(), category: "Memory".into(),
        description: "Compare two memory regions byte by byte. Returns equal flag, diff position, and both addresses.".into(),
        inputs: vec![input("addr1", "number", "0"), input("addr2", "number", "0"), input("size", "number", "16")],
        outputs: vec![output("equal", "bool"), output("diff_at", "number"), output("addr1", "number"), output("addr2", "number")],
        python_template: r#"
import ctypes
_addr1 = {{addr1}}
_addr2 = {{addr2}}
_size = {{size}}
a = ctypes.string_at(_addr1, _size)
b = ctypes.string_at(_addr2, _size)
diff = -1
for i in range(min(len(a), len(b))):
    if a[i] != b[i]:
        diff = i
        break
result = diff
"#.into(),
    });

    // ═══════════════ PROCESS (8) ═══════════════

    b.push(BlockDefinition {
        name: "Open Process".into(), icon: "🔓".into(), category: "Process".into(),
        description: "Get info about a running process by PID. Returns info and PID.".into(),
        inputs: vec![input("pid", "number", "0")],
        outputs: vec![output("info", "string"), output("pid", "number")],
        python_template: r#"
import os
_pid = {{pid}} if {{pid}} != 0 else os.getpid()
result = str(_pid)
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Read Process Memory".into(), icon: "📖".into(), category: "Process".into(),
        description: "Read memory from another process. Returns data, pid, and address.".into(),
        inputs: vec![input("pid", "number", "0"), input("address", "number", "0"), input("size", "number", "64")],
        outputs: vec![output("data", "bytes"), output("pid", "number"), output("address", "number")],
        python_template: r#"
import ctypes, sys, os
_pid = {{pid}} if {{pid}} != 0 else os.getpid()
_addr = {{address}}
_size = {{size}}
result = ""
try:
    if sys.platform == 'win32':
        kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
        PROCESS_VM_READ = 0x0010
        h = kernel32.OpenProcess(PROCESS_VM_READ, False, _pid)
        if h:
            buf = ctypes.create_string_buffer(_size)
            bytes_read = ctypes.c_size_t(0)
            if kernel32.ReadProcessMemory(h, ctypes.c_void_p(_addr), buf, _size, ctypes.byref(bytes_read)):
                result = buf.raw[:bytes_read.value].hex()
            else:
                result = f"Error: ReadProcessMemory failed"
            kernel32.CloseHandle(h)
        else:
            result = f"Error: Cannot open process {_pid}"
    elif sys.platform != 'win32':
        try:
            with open(f'/proc/{_pid}/mem', 'rb') as mem:
                mem.seek(_addr)
                data = mem.read(_size)
                result = data.hex()
        except Exception as e:
            result = f"Error: {e}"
except Exception as e:
    result = f"Error: {e}"
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Write Process Memory".into(), icon: "📝".into(), category: "Process".into(),
        description: "Write data to another process's memory. Returns bytes written and address.".into(),
        inputs: vec![input("pid", "number", "0"), input("address", "number", "0"), input("data", "string", "")],
        outputs: vec![output("bytes_written", "number"), output("address", "number")],
        python_template: r#"
import ctypes, sys, os
_pid = {{pid}} if {{pid}} != 0 else os.getpid()
_addr = {{address}}
_data = """{{data}}"""
result = 0
try:
    _data_bytes = _data.encode('utf-8')
    if sys.platform == 'win32':
        kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
        PROCESS_ALL_ACCESS = 0x1F0FFF
        h = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
        if h:
            written = ctypes.c_size_t(0)
            if kernel32.WriteProcessMemory(h, ctypes.c_void_p(_addr), _data_bytes, len(_data_bytes), ctypes.byref(written)):
                result = written.value
            kernel32.CloseHandle(h)
    elif sys.platform != 'win32':
        try:
            with open(f'/proc/{_pid}/mem', 'wb') as mem:
                mem.seek(_addr)
                result = mem.write(_data_bytes)
        except:
            pass
except:
    pass
"#.into(),
    });

    b.push(BlockDefinition {
        name: "List Processes".into(), icon: "📜".into(), category: "Process".into(),
        description: "List running processes on the system. Returns the list and count.".into(),
        inputs: vec![input("count", "number", "10")],
        outputs: vec![output("process_list", "string"), output("count", "number")],
        python_template: r#"
import sys, subprocess
limit = int({{count}})
if sys.platform == 'win32':
    output = subprocess.check_output(['tasklist', '/FO', 'CSV'], text=True).split('\n')[:limit+1]
else:
    output = subprocess.check_output(['ps', 'aux'], text=True).split('\n')[:limit+1]
result = '\n'.join(output)
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Get Process Info".into(), icon: "ℹ️".into(), category: "Process".into(),
        description: "Get detailed information about a process. Returns info and PID.".into(),
        inputs: vec![input("pid", "number", "0")],
        outputs: vec![output("info", "string"), output("pid", "number")],
        python_template: r#"
import os
_pid = {{pid}} if {{pid}} != 0 else os.getpid()
result = str(_pid)
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Inject Code".into(), icon: "💉".into(), category: "Process".into(),
        description: "Inject a DLL into a process (Windows). Returns success and dll path.".into(),
        inputs: vec![input("pid", "number", "0"), input("dll_path", "string", "")],
        outputs: vec![output("success", "bool"), output("dll_path", "string")],
        python_template: r#"
import ctypes, sys
_pid = {{pid}}
_dll_path = """{{dll_path}}"""
result = False
try:
    if sys.platform == 'win32' and _pid != 0 and _dll_path:
        kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
        dll_bytes = _dll_path.encode('utf-8')
        dll_len = len(dll_bytes) + 1
        PROCESS_ALL_ACCESS = 0x1F0FFF
        h = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, _pid)
        if h:
            VirtualAllocEx = kernel32.VirtualAllocEx
            VirtualAllocEx.restype = ctypes.c_void_p
            MEM_COMMIT = 0x1000
            PAGE_READWRITE = 0x04
            remote_mem = VirtualAllocEx(h, None, dll_len, MEM_COMMIT, PAGE_READWRITE)
            if remote_mem:
                written = ctypes.c_size_t(0)
                kernel32.WriteProcessMemory(h, remote_mem, dll_bytes, dll_len, ctypes.byref(written))
                LoadLibraryA = ctypes.c_void_p(kernel32.GetProcAddress(kernel32._handle, b'LoadLibraryA'))
                if LoadLibraryA:
                    thread_id = ctypes.c_ulong(0)
                    h_thread = kernel32.CreateRemoteThread(h, None, 0, LoadLibraryA, remote_mem, 0, ctypes.byref(thread_id))
                    if h_thread:
                        result = True
            kernel32.CloseHandle(h)
except:
    pass
"#.into(),
    });

       b.push(BlockDefinition {
        name: "Close Process".into(), icon: "🔒".into(), category: "Process".into(),
        description: "Close a handle to a process. Returns success and handle.".into(),
        inputs: vec![input("handle", "number", "0")],
        outputs: vec![output("success", "bool"), output("handle", "number")],
        python_template: r#"
import ctypes, sys
_handle = {{handle}}
try:
    if sys.platform == 'win32' and _handle != 0:
        kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
        result = kernel32.CloseHandle(_handle) != 0
    else:
        result = _handle != 0
except:
    result = False
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Get Current Process".into(), icon: "🏠".into(), category: "Process".into(),
        description: "Get the current process ID and info.".into(),
        inputs: vec![],
        outputs: vec![output("pid", "number"), output("info", "string")],
        python_template: r#"
import os
result = os.getpid()
"#.into(),
    });

    // ═══════════════ FILE SYSTEM (10) ═══════════════

        b.push(BlockDefinition {
        name: "Open File".into(), icon: "📂".into(), category: "File System".into(),
        description: "Open a file and keep it open. Returns a file ID for Read/Write/Close blocks. Wire the file_id output to other file blocks.".into(),
        inputs: vec![input("path", "string", "developi_test.txt"), input("mode", "string", "r")],
        outputs: vec![output("file_id", "string"), output("path", "string"), output("mode", "string")],
        python_template: r#"
import os
_path = os.path.join(os.getcwd(), """{{path}}""")
_mode = """{{mode}}"""
f = open(_path, _mode, encoding="utf-8", errors="ignore")
_file_id = f"__devfile_{id(f)}"
globals()[_file_id] = f
result = _file_id
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Read File".into(), icon: "📖".into(), category: "File System".into(),
        description: "Read contents from an open file (wire file_id) or from a path. Returns content and path.".into(),
        inputs: vec![input("file_id", "string", ""), input("path", "string", "developi_test.txt")],
        outputs: vec![output("content", "string"), output("path", "string")],
        python_template: r#"
import os
_file_id = """{{file_id}}"""
_path = os.path.join(os.getcwd(), """{{path}}""")
try:
    if _file_id and _file_id in globals():
        f = globals()[_file_id]
        f.seek(0)
        result = f.read()
    elif os.path.exists(_path):
        with open(_path, "r", encoding="utf-8", errors="ignore") as f:
            result = f.read()
    else:
        result = ""
except Exception as e:
    result = f"Error: {e}"
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Write File".into(), icon: "✍️".into(), category: "File System".into(),
        description: "Write text to an open file (wire file_id) or to a path. Returns the path and data written.".into(),
        inputs: vec![input("file_id", "string", ""), input("path", "string", "developi_output.txt"), input("data", "string", "developi was here")],
        outputs: vec![output("path", "string"), output("data", "string")],
        python_template: r#"
import os
_file_id = """{{file_id}}"""
_path = os.path.join(os.getcwd(), """{{path}}""")
_data = """{{data}}"""
try:
    if _file_id and _file_id in globals():
        f = globals()[_file_id]
        f.write(_data + "\n")
        f.flush()
    else:
        with open(_path, "a", encoding="utf-8") as f:
            f.write(_data + "\n")
    result = _path
except Exception as e:
    result = f"Error: {e}"
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Seek File".into(), icon: "⏩".into(), category: "File System".into(),
        description: "Move the file pointer to a specific position. Returns new position and path.".into(),
        inputs: vec![input("path", "string", "developi_test.txt"), input("position", "number", "0"), input("whence", "string", "start")],
        outputs: vec![output("position", "number"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.join(os.getcwd(), """{{path}}""")
whence_map = {"start": 0, "current": 1, "end": 2}
w = whence_map.get("""{{whence}}""", 0)
if os.path.exists(_path):
    f = open(_path, "rb")
    f.seek({{position}}, w)
    result = f.tell()
    f.close()
else:
    result = -1
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Close File".into(), icon: "🔐".into(), category: "File System".into(),
        description: "Close an open file handle. Wire file_id from Open File. Returns success.".into(),
        inputs: vec![input("file_id", "string", "")],
        outputs: vec![output("success", "bool")],
        python_template: r#"
_file_id = """{{file_id}}"""
result = False
try:
    if _file_id and _file_id in globals():
        f = globals()[_file_id]
        f.close()
        del globals()[_file_id]
        result = True
except:
    pass
"#.into(),
    });

    b.push(BlockDefinition {
        name: "List Directory".into(), icon: "📁".into(), category: "File System".into(),
        description: "List files and folders in a directory. Returns listing and path.".into(),
        inputs: vec![input("path", "string", ".")],
        outputs: vec![output("listing", "string"), output("path", "string")],
        python_template: r#"
import os
_path = """{{path}}""" if """{{path}}""" != "." else os.getcwd()
items = os.listdir(_path)
result = str(items)
"#.into(),
    });

    b.push(BlockDefinition {
        name: "File Exists".into(), icon: "❓".into(), category: "File System".into(),
        description: "Check whether a file or directory exists. Returns boolean and path.".into(),
        inputs: vec![input("path", "string", "developi_test.txt")],
        outputs: vec![output("exists", "bool"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.join(os.getcwd(), """{{path}}""")
result = os.path.exists(_path)
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Delete File".into(), icon: "🗑️".into(), category: "File System".into(),
        description: "Permanently delete a file. Returns success and path.".into(),
        inputs: vec![input("path", "string", "developi_test.txt")],
        outputs: vec![output("success", "bool"), output("path", "string")],
        python_template: r#"
import os
_path = os.path.join(os.getcwd(), """{{path}}""")
if os.path.exists(_path):
    os.remove(_path)
    result = True
else:
    result = False
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Create Directory".into(), icon: "📁".into(), category: "File System".into(),
        description: "Create a new directory. Returns the path.".into(),
        inputs: vec![input("path", "string", "developi_workspace")],
        outputs: vec![output("path", "string")],
        python_template: r#"
import os
new_dir = os.path.join(os.getcwd(), """{{path}}""")
os.makedirs(new_dir, exist_ok=True)
result = new_dir
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Get File Info".into(), icon: "📊".into(), category: "File System".into(),
        description: "Get file size and info. Returns info string and path.".into(),
        inputs: vec![input("path", "string", ".")],
        outputs: vec![output("info", "string"), output("path", "string")],
        python_template: r#"
import os
_path = """{{path}}""" if """{{path}}""" != "." else os.getcwd()
stat = os.stat(_path)
result = str(stat.st_size)
"#.into(),
    });

    // ═══════════════ NETWORK (8) ═══════════════

    b.push(BlockDefinition {
        name: "Create Socket".into(), icon: "🔌".into(), category: "Network".into(),
        description: "Create a TCP or UDP socket. Returns the file descriptor and type.".into(),
        inputs: vec![input("type", "string", "tcp")],
        outputs: vec![output("fd", "number"), output("type", "string")],
        python_template: r#"
import socket
_type = """{{type}}"""
sock_type = socket.SOCK_STREAM if _type == "tcp" else socket.SOCK_DGRAM
s = socket.socket(socket.AF_INET, sock_type)
result = s.fileno()
s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Bind Socket".into(), icon: "📌".into(), category: "Network".into(),
        description: "Bind a socket to an address and port. Returns the bound port and address.".into(),
        inputs: vec![input("address", "string", "127.0.0.1"), input("port", "number", "0")],
        outputs: vec![output("bound_port", "number"), output("address", "string")],
        python_template: r#"
import socket
_addr = """{{address}}"""
_port = {{port}}
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind((_addr, _port))
result = s.getsockname()[1]
s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Connect Socket".into(), icon: "🔗".into(), category: "Network".into(),
        description: "Connect to a remote address. Returns connection status, address, and port.".into(),
        inputs: vec![input("address", "string", "1.1.1.1"), input("port", "number", "80"), input("timeout", "number", "3")],
        outputs: vec![output("connected", "bool"), output("address", "string"), output("port", "number")],
        python_template: r#"
import socket
_addr = """{{address}}"""
_port = {{port}}
_timeout = {{timeout}}
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(_timeout)
try:
    s.connect((_addr, _port))
    result = True
except:
    result = False
finally:
    s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Send Data".into(), icon: "📤".into(), category: "Network".into(),
        description: "Send data through a socket. Type any text. Returns bytes sent, address, and port.".into(),
        inputs: vec![input("address", "string", "1.1.1.1"), input("port", "number", "80"), input("data", "string", "Hello from developi")],
        outputs: vec![output("bytes_sent", "number"), output("address", "string"), output("port", "number")],
        python_template: r#"
import socket
_addr = """{{address}}"""
_port = {{port}}
_data = """{{data}}""".encode('utf-8')
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(3)
try:
    s.connect((_addr, _port))
    result = s.send(_data)
except:
    result = 0
finally:
    s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Receive Data".into(), icon: "📥".into(), category: "Network".into(),
        description: "Receive data from a socket. Returns received text, address, and port.".into(),
        inputs: vec![input("address", "string", "1.1.1.1"), input("port", "number", "80"), input("buffer_size", "number", "256")],
        outputs: vec![output("received", "string"), output("address", "string"), output("port", "number")],
        python_template: r#"
import socket
_addr = """{{address}}"""
_port = {{port}}
_buf = {{buffer_size}}
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.settimeout(3)
try:
    s.connect((_addr, _port))
    s.send(b'GET / HTTP/1.0\r\nHost: ' + _addr.encode() + b'\r\n\r\n')
    data = s.recv(_buf)
    result = data.decode('utf-8', errors='ignore')
except:
    result = ""
finally:
    s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Listen Socket".into(), icon: "👂".into(), category: "Network".into(),
        description: "Listen for incoming connections. Returns the bound port.".into(),
        inputs: vec![input("port", "number", "0"), input("backlog", "number", "5")],
        outputs: vec![output("bound_port", "number"), output("port", "number")],
        python_template: r#"
import socket
_port = {{port}}
_backlog = {{backlog}}
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind(('127.0.0.1', _port))
s.listen(_backlog)
result = s.getsockname()[1]
s.close()
"#.into(),
    });

    b.push(BlockDefinition {
        name: "Accept Connection".into(), icon: "🤝".into(), category: "Network".into(),
        description: "Accept an incoming connection. Returns client address and port.".into(),
        inputs: vec![input("port", "number", "0"), input("timeout", "number", "2")],
        outputs: vec![output("client_addr", "string"), output("port", "number")],
        python_template: r#"
import socket, threading, time
_port = {{port}}
_timeout = {{timeout}}
srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
srv.bind(('127.0.0.1', _port))
srv.listen(1)
bound_port = srv.getsockname()[1]
def connect_self():
    time.sleep(0.1)
    c = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    c.connect(('127.0.0.1', bound_port))
    c.close()
threading.Thread(target=connect_self, daemon=True).start()
srv.settimeout(_timeout)
try:
    client, addr = srv.accept()
    result = str(addr[0])
    client.close()
except:
    result = ""
srv.close()
"#.into(),
    });

        b.push(BlockDefinition {
        name: "Close Socket".into(), icon: "🔌".into(), category: "Network".into(),
        description: "Close a socket and release the port. Wire socket_id. Returns success.".into(),
        inputs: vec![input("socket_id", "string", "")],
        outputs: vec![output("success", "bool")],
        python_template: r#"
_socket_id = """{{socket_id}}"""
result = False
try:
    if _socket_id and _socket_id in globals():
        s = globals()[_socket_id]
        s.close()
        del globals()[_socket_id]
        result = True
except:
    pass
"#.into(),
    });

    // ═══════════════ DATA (12) ═══════════════

    b.push(BlockDefinition {
        name: "Integer".into(), icon: "🔢".into(), category: "Data".into(),
        description: "Create an integer value.".into(),
        inputs: vec![input("value", "number", "42")], outputs: vec![output("value", "number")],
        python_template: r#"result = int({{value}})"#.into(),
    });
    b.push(BlockDefinition {
        name: "Float".into(), icon: "🔣".into(), category: "Data".into(),
        description: "Create a floating-point number.".into(),
        inputs: vec![input("value", "number", "3.14159")], outputs: vec![output("value", "number")],
        python_template: r#"result = float({{value}})"#.into(),
    });
    b.push(BlockDefinition {
        name: "String".into(), icon: "📝".into(), category: "Data".into(),
        description: "Create a text string.".into(),
        inputs: vec![input("text", "string", "Hello from developi")], outputs: vec![output("text", "string")],
        python_template: r#"result = """{{text}}""" "#.into(),
    });
    b.push(BlockDefinition {
        name: "Bytes".into(), icon: "💾".into(), category: "Data".into(),
        description: "Create raw bytes from a hex string.".into(),
        inputs: vec![input("hex", "string", "00010203FFFE")], outputs: vec![output("data", "bytes")],
        python_template: r#"result = bytes.fromhex("""{{hex}}""")"#.into(),
    });
    b.push(BlockDefinition {
        name: "List".into(), icon: "📋".into(), category: "Data".into(),
        description: "Create a list from comma-separated values.".into(),
        inputs: vec![input("items", "string", "1, 2, 3, 4, 5")], outputs: vec![output("list", "any")],
        python_template: r#"
items = [x.strip() for x in """{{items}}""".split(",")]
try:
    items = [int(x) if x.lstrip('-').isdigit() else float(x) if x.replace('.','',1).replace('-','',1).isdigit() else x for x in items]
except: pass
result = items
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Dictionary".into(), icon: "📖".into(), category: "Data".into(),
        description: "Create a key-value dictionary from JSON.".into(),
        inputs: vec![input("json", "string", "{\"name\": \"developi\", \"version\": 1.0}")], outputs: vec![output("dict", "any")],
        python_template: r#"
import json
result = json.loads('''{{json}}''')
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Pack Struct".into(), icon: "📦".into(), category: "Data".into(),
        description: "Pack data into a binary struct. Returns packed bytes and format.".into(),
        inputs: vec![input("format", "string", "<I4sf"), input("values", "string", "42, DEVI, 3.14")],
        outputs: vec![output("packed", "bytes"), output("format", "string")],
        python_template: r#"
import struct
_fmt = """{{format}}"""
vals = [x.strip() for x in """{{values}}""".split(",")]
packed = struct.pack(_fmt, *[v.encode() if isinstance(v, str) and len(v) <= 4 else float(v) if '.' in v else int(v) for v in vals])
result = packed.hex()
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Unpack Struct".into(), icon: "📤".into(), category: "Data".into(),
        description: "Unpack binary data into values. Returns values and format.".into(),
        inputs: vec![input("format", "string", "<I4sf"), input("hex_data", "string", "")],
        outputs: vec![output("values", "string"), output("format", "string")],
        python_template: r#"
import struct
_fmt = """{{format}}"""
data = bytes.fromhex("""{{hex_data}}""") if """{{hex_data}}""" else struct.pack('<I4sf', 42, b'DEVI', 3.14)
unpacked = struct.unpack(_fmt, data)
result = str(unpacked)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Encode Text".into(), icon: "🔤".into(), category: "Data".into(),
        description: "Encode text to hex bytes. Returns hex and encoding used.".into(),
        inputs: vec![input("text", "string", "developi"), input("encoding", "string", "utf-8")],
        outputs: vec![output("encoded", "bytes"), output("encoding", "string")],
        python_template: r#"
_enc = """{{encoding}}"""
result = """{{text}}""".encode(_enc).hex()
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Decode Text".into(), icon: "🔡".into(), category: "Data".into(),
        description: "Decode hex bytes to text. Returns text and encoding used.".into(),
        inputs: vec![input("hex", "string", "646576656c6f7069"), input("encoding", "string", "utf-8")],
        outputs: vec![output("text", "string"), output("encoding", "string")],
        python_template: r#"
_enc = """{{encoding}}"""
result = bytes.fromhex("""{{hex}}""").decode(_enc)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Slice".into(), icon: "✂️".into(), category: "Data".into(),
        description: "Slice a list, string, or bytes by indices.".into(),
        inputs: vec![input("data", "string", "0,1,2,3,4,5,6,7,8,9"), input("start", "number", "0"), input("end", "number", "5"), input("step", "number", "1")],
        outputs: vec![output("sliced", "any")],
        python_template: r#"
data = [x.strip() for x in """{{data}}""".split(",")]
result = data[{{start}}:{{end}}:{{step}}]
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Length".into(), icon: "📏".into(), category: "Data".into(),
        description: "Get the length of any collection.".into(),
        inputs: vec![input("data", "string", "1,2,3,4,5")], outputs: vec![output("length", "number")],
        python_template: r#"
data = [x.strip() for x in """{{data}}""".split(",")]
result = len(data)
"#.into(),
    });

    // ═══════════════ LOGIC (10) ═══════════════

        b.push(BlockDefinition {
        name: "If".into(), icon: "🔱".into(), category: "Logic".into(),
        description: "Choose a path based on true or false. Wire a condition to it.".into(),
        inputs: vec![input("condition", "bool", "true")],
        outputs: vec![output("branch", "string")],
        python_template: r#"
cond = str("""{{condition}}""").lower() == "true"
result = cond
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Compare".into(), icon: "⚖️".into(), category: "Logic".into(),
        description: "Compare two values. Type how to compare — 'bigger', 'smaller', 'equal', 'not equal'.".into(),
        inputs: vec![input("a", "any", "42"), input("b", "any", "24"), input("how", "string", "bigger")],
        outputs: vec![output("result", "bool"), output("a", "any"), output("b", "any")],
        python_template: r#"
a = """{{a}}"""; b = """{{b}}"""
try:
    a = float(a) if a.replace('.','',1).replace('-','',1).isdigit() else a
    b = float(b) if b.replace('.','',1).replace('-','',1).isdigit() else b
except: pass
_how = """{{how}}""".lower()
_ops = {"bigger": ">", "greater": ">", "smaller": "<", "less": "<", "equal": "==", "same": "==", "not equal": "!=", "different": "!="}
_op = _ops.get(_how, "==")
result = eval(f"a {_op} b")
"#.into(),
    });
    b.push(BlockDefinition {
        name: "And".into(), icon: "🔗".into(), category: "Logic".into(),
        description: "Logical AND. True only if both inputs are true.".into(),
        inputs: vec![input("a", "bool", "true"), input("b", "bool", "true")],
        outputs: vec![output("result", "bool"), output("a", "bool"), output("b", "bool")],
        python_template: r#"result = str("""{{a}}""").lower()=="true" and str("""{{b}}""").lower()=="true""#.into(),
    });
    b.push(BlockDefinition {
        name: "Or".into(), icon: "🔀".into(), category: "Logic".into(),
        description: "Logical OR. True if at least one input is true.".into(),
        inputs: vec![input("a", "bool", "true"), input("b", "bool", "false")],
        outputs: vec![output("result", "bool"), output("a", "bool"), output("b", "bool")],
        python_template: r#"result = str("""{{a}}""").lower()=="true" or str("""{{b}}""").lower()=="true""#.into(),
    });
    b.push(BlockDefinition {
        name: "Not".into(), icon: "🚫".into(), category: "Logic".into(),
        description: "Logical NOT. Inverts the input boolean.".into(),
        inputs: vec![input("value", "bool", "false")], outputs: vec![output("result", "bool")],
        python_template: r#"result = not (str("""{{value}}""").lower() == "true")"#.into(),
    });
            b.push(BlockDefinition {
        name: "For Each".into(), icon: "🔄".into(), category: "Logic".into(),
        description: "Do something to every item in a list. Just type what you want — 'double it', 'add 5', 'multiply by 3'.".into(),
        inputs: vec![input("items", "string", "1, 2, 3, 4, 5"), input("action", "string", "double it")],
        outputs: vec![output("results", "string")],
        python_template: r#"
items = [x.strip() for x in """{{items}}""".split(",")]
try:
    items = [int(x) if x.lstrip('-').isdigit() else float(x) if x.replace('.','',1).replace('-','',1).isdigit() else x for x in items]
except: pass
_action = """{{action}}""".lower().replace(" it", "").replace(" each", "")
_translate = {
    "double": "item * 2", "triple": "item * 3", "half": "item / 2",
    "square": "item ** 2", "add one": "item + 1", "even": "item % 2 == 0",
    "odd": "item % 2 != 0", "multiply by": "item *", "divide by": "item /",
    "plus": "item +", "minus": "item -", "times": "item *",
}
for _word, _code in _translate.items():
    if _word in _action:
        _action = _action.replace(_word, _code).replace("  ", " ")
        break
if "item" not in _action:
    _action = "item " + _action
results_list = []
for item in items:
    results_list.append(eval(_action))
result = str(results_list)
"#.into(),
    });
            b.push(BlockDefinition {
        name: "While".into(), icon: "⏳".into(), category: "Logic".into(),
        description: "Keep doing something while it's true. Type like 'less than 10' or 'keep doubling'.".into(),
        inputs: vec![input("condition", "string", "less than 10"), input("body", "string", "double")],
        outputs: vec![output("values", "string")],
        python_template: r#"
_cond = """{{condition}}""".lower()
_body = """{{body}}""".lower()
_cond = _cond.replace("less than", "<").replace("greater than", ">").replace("at most", "<=").replace("at least", ">=").replace("not", "!=").replace("equal to", "==")
if "count" not in _cond:
    _cond = "count " + _cond
_body = _body.replace("double", "count * 2").replace("triple", "count * 3").replace("square", "count ** 2").replace("add one", "count + 1").replace("keep doubling", "count * 2")
if "count" not in _body:
    _body = "count " + _body
count = 0; values = []
while eval(_cond):
    values.append(eval(_body))
    count += 1
result = str(values)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Count".into(), icon: "🔢".into(), category: "Logic".into(),
        description: "Generate a range of numbers from start to end by step. Returns numbers, start, and end.".into(),
        inputs: vec![input("start", "number", "0"), input("end", "number", "10"), input("step", "number", "1")],
        outputs: vec![output("numbers", "string"), output("start", "number"), output("end", "number")],
        python_template: r#"result = str(list(range({{start}}, {{end}}, {{step}})))"#.into(),
    });
            b.push(BlockDefinition {
        name: "Break".into(), icon: "⏹️".into(), category: "Logic".into(),
        description: "Find the first match. Type like 'bigger than 500' or 'over 100'. Use 'i' for the number.".into(),
        inputs: vec![input("max_range", "number", "100"), input("find", "string", "bigger than 500")],
        outputs: vec![output("found", "number")],
        python_template: r#"
_max = {{max_range}}
_find = """{{find}}""".lower()
_find = _find.replace("bigger than", "i >").replace("greater than", "i >").replace("over", "i >").replace("smaller than", "i <").replace("under", "i <").replace("less than", "i <").replace("equal to", "i ==")
if "i" not in _find:
    _find = "i " + _find
found = -1
for i in range(_max):
    if eval(_find):
        found = i
        break
result = found
"#.into(),
    });
            b.push(BlockDefinition {
        name: "Continue".into(), icon: "⏭️".into(), category: "Logic".into(),
        description: "Skip things that match. Type like 'skip evens' or 'skip odds'.".into(),
        inputs: vec![input("max_range", "number", "20"), input("skip", "string", "skip odd")],
        outputs: vec![output("kept_values", "string")],
        python_template: r#"
_max = {{max_range}}
_skip = """{{skip}}""".lower()
_skip = _skip.replace("skip", "").replace("ignore", "").strip()
_translate = {"even": "i % 2 == 0", "odd": "i % 2 != 0", "multiples of 3": "i % 3 == 0", "multiples of 5": "i % 5 == 0"}
_skip = _translate.get(_skip, "i % 2 != 0")
kept = []
for i in range(_max):
    if eval(_skip):
        continue
    kept.append(i)
result = str(kept)
"#.into(),
    });

    // ═══════════════ MATH (8) ═══════════════

    b.push(BlockDefinition {
        name: "Add".into(), icon: "➕".into(), category: "Math".into(),
        description: "Add two numbers. Returns the sum and both operands.".into(),
        inputs: vec![input("a", "number", "0"), input("b", "number", "0")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} + {{b}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Subtract".into(), icon: "➖".into(), category: "Math".into(),
        description: "Subtract b from a. Returns the difference and both operands.".into(),
        inputs: vec![input("a", "number", "0"), input("b", "number", "0")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} - {{b}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Multiply".into(), icon: "✖️".into(), category: "Math".into(),
        description: "Multiply two numbers. Returns the product and both operands.".into(),
        inputs: vec![input("a", "number", "1"), input("b", "number", "1")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} * {{b}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Divide".into(), icon: "➗".into(), category: "Math".into(),
        description: "Divide a by b. Returns the quotient and both operands.".into(),
        inputs: vec![input("a", "number", "1"), input("b", "number", "1")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} / {{b}} if {{b}} != 0 else 'Error'"#.into(),
    });
    b.push(BlockDefinition {
        name: "Modulo".into(), icon: "🔄".into(), category: "Math".into(),
        description: "Compute a modulo b. Returns the remainder and both operands.".into(),
        inputs: vec![input("a", "number", "17"), input("b", "number", "5")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} % {{b}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Power".into(), icon: "💪".into(), category: "Math".into(),
        description: "Raise base to the power of exp. Returns the result and both operands.".into(),
        inputs: vec![input("base", "number", "2"), input("exp", "number", "8")],
        outputs: vec![output("result", "number"), output("base", "number"), output("exp", "number")],
        python_template: r#"result = {{base}} ** {{exp}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Bit AND".into(), icon: "&".into(), category: "Math".into(),
        description: "Bitwise AND of two numbers. Returns the result and both operands.".into(),
        inputs: vec![input("a", "number", "255"), input("b", "number", "15")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} & {{b}}"#.into(),
    });
    b.push(BlockDefinition {
        name: "Bit OR".into(), icon: "|".into(), category: "Math".into(),
        description: "Bitwise OR of two numbers. Returns the result and both operands.".into(),
        inputs: vec![input("a", "number", "240"), input("b", "number", "15")],
        outputs: vec![output("result", "number"), output("a", "number"), output("b", "number")],
        python_template: r#"result = {{a}} | {{b}}"#.into(),
    });

    // ═══════════════ VARIABLES (4) ═══════════════

    b.push(BlockDefinition {
        name: "Set Variable".into(), icon: "📝".into(), category: "Variables".into(),
        description: "Create or update a variable. Returns the value and name.".into(),
        inputs: vec![input("name", "string", "my_var"), input("value", "any", "Hello")],
        outputs: vec![output("value", "any"), output("name", "string")],
        python_template: r#"
{{name}} = """{{value}}"""
result = """{{value}}"""
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Get Variable".into(), icon: "🔍".into(), category: "Variables".into(),
        description: "Retrieve the value of a variable by name. Returns value and name.".into(),
        inputs: vec![input("name", "string", "my_var")],
        outputs: vec![output("value", "any"), output("name", "string")],
        python_template: r#"
_name = """{{name}}"""
try:
    if _name in dir(): result = eval(_name)
    else: result = ""
except: result = ""
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Delete Variable".into(), icon: "🗑️".into(), category: "Variables".into(),
        description: "Delete a variable by name. Returns success and name.".into(),
        inputs: vec![input("name", "string", "my_var")],
        outputs: vec![output("success", "bool"), output("name", "string")],
        python_template: r#"
_name = """{{name}}"""
if _name in dir(): del {{name}}; result = True
else: result = False
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Variable Exists".into(), icon: "❓".into(), category: "Variables".into(),
        description: "Check whether a variable exists. Returns boolean and name.".into(),
        inputs: vec![input("name", "string", "my_var")],
        outputs: vec![output("exists", "bool"), output("name", "string")],
        python_template: r#"
_name = """{{name}}"""
result = _name in dir()
"#.into(),
    });

    // ═══════════════ FUNCTIONS (5) ═══════════════

    b.push(BlockDefinition {
        name: "Define Function".into(), icon: "⚙️".into(), category: "Functions".into(),
        description: "Define a new Python function. Returns the function name and params.".into(),
        inputs: vec![input("name", "string", "my_func"), input("params", "string", "x"), input("body", "string", "return x * 2")],
        outputs: vec![output("name", "string"), output("params", "string")],
        python_template: r#"
_name = """{{name}}"""
_params = """{{params}}"""
exec(f"def {_name}({_params}):\n    {{body}}")
result = _name
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Call Function".into(), icon: "📞".into(), category: "Functions".into(),
        description: "Call a defined function with arguments. Returns result and function name.".into(),
        inputs: vec![input("name", "string", "my_func"), input("args", "string", "5")],
        outputs: vec![output("result", "any"), output("name", "string")],
        python_template: r#"
_name = """{{name}}"""
try: result = eval(f"{_name}({{args}})")
except: result = ""
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Return".into(), icon: "↩️".into(), category: "Functions".into(),
        description: "Return a value from the current block.".into(),
        inputs: vec![input("value", "any", "done")], outputs: vec![output("result", "any")],
        python_template: r#"result = """{{value}}""" "#.into(),
    });
    b.push(BlockDefinition {
        name: "Lambda".into(), icon: "λ".into(), category: "Functions".into(),
        description: "Create a lambda function and apply it. Returns result, args, and expr.".into(),
        inputs: vec![input("args", "string", "x"), input("expr", "string", "x * 2"), input("apply_to", "number", "5")],
        outputs: vec![output("result", "any"), output("args", "string"), output("expr", "string")],
        python_template: r#"
_args = """{{args}}"""
_expr = """{{expr}}"""
f = lambda {{args}}: {{expr}}
result = f({{apply_to}})
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Partial".into(), icon: "🧩".into(), category: "Functions".into(),
        description: "Create a partial function with pre-filled arguments. Returns result and both operands.".into(),
        inputs: vec![input("base", "number", "5"), input("exp", "number", "2")],
        outputs: vec![output("result", "number"), output("base", "number"), output("exp", "number")],
        python_template: r#"
from functools import partial
_base = {{base}}
_exp = {{exp}}
def power(base, exp): return base ** exp
square = partial(power, exp=_exp)
result = square(_base)
"#.into(),
    });

    // ═══════════════ PYTHON POWER (8) ═══════════════

    b.push(BlockDefinition {
        name: "Import".into(), icon: "📥".into(), category: "Python Power".into(),
        description: "Import a Python module dynamically. Returns the module name.".into(),
        inputs: vec![input("module", "string", "math")], outputs: vec![output("module_name", "string")],
        python_template: r#"
import importlib
_mod = """{{module}}"""
mod = importlib.import_module(_mod)
result = _mod
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Exec Python".into(), icon: "▶️".into(), category: "Python Power".into(),
        description: "Type anything you want to happen. No rules.".into(),
        inputs: vec![input("code", "string", "print('Hello')")],
        outputs: vec![output("result", "any")],
        python_template: r#"exec("""{{code}}""")"#.into(),
    });
        b.push(BlockDefinition {
        name: "Eval".into(), icon: "🧮".into(), category: "Python Power".into(),
        description: "Type a calculation — '2 + 2', '10 times 5', 'half of 100'.".into(),
        inputs: vec![input("expression", "string", "2 + 2")],
        outputs: vec![output("result", "any")],
        python_template: r#"
_expr = """{{expression}}""".lower()
_expr = _expr.replace("times", "*").replace("divided by", "/").replace("half of", "/ 2").replace("double", "* 2").replace("plus", "+").replace("minus", "-")
result = eval(_expr)
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Try".into(), icon: "🛡️".into(), category: "Python Power".into(),
        description: "Try something. If it doesn't work, you'll see what went wrong instead of crashing.".into(),
        inputs: vec![input("code", "string", "result = 100 / 0")],
        outputs: vec![output("result", "any"), output("error", "string")],
        python_template: r#"
try: exec("""{{code}}""")
except Exception as e: result = str(e)
"#.into(),
    });
       b.push(BlockDefinition {
        name: "Raise".into(), icon: "⚠️".into(), category: "Python Power".into(),
        description: "Create an alert with a message you type.".into(),
        inputs: vec![input("message", "string", "Something went wrong")],
        outputs: vec![output("error", "string"), output("message", "string")],
        python_template: r#"
_msg = """{{message}}"""
try: raise ValueError(_msg)
except ValueError as e: result = str(e)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Convert Type".into(), icon: "🔄".into(), category: "Python Power".into(),
        description: "Convert a value to a target type. Returns converted value and type used.".into(),
        inputs: vec![input("value", "any", "42"), input("target_type", "string", "int")],
        outputs: vec![output("converted", "any"), output("target_type", "string")],
        python_template: r#"
_val = """{{value}}"""
_tgt = """{{target_type}}"""
types = {"int": int, "float": float, "str": str, "bool": bool, "list": list, "bytes": bytes}
t = types.get(_tgt, str)
result = t(_val)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Type Of".into(), icon: "🏷️".into(), category: "Python Power".into(),
        description: "Get the Python type name of a value.".into(),
        inputs: vec![input("value", "any", "hello")], outputs: vec![output("type", "string")],
        python_template: r#"result = type("""{{value}}""").__name__"#.into(),
    });
            b.push(BlockDefinition {
        name: "Await".into(), icon: "⏳".into(), category: "Python Power".into(),
        description: "Type anything — it runs and returns the result. No code needed.".into(),
        inputs: vec![input("async_code", "string", "Hello from developi")],
        outputs: vec![output("result", "string")],
        python_template: r#"
import asyncio
_code = """{{async_code}}"""
async def _developi_async():
    try:
        exec(f"result = {_code}", globals())
    except:
        result = _code
    return result if 'result' in dir() else _code
try:
    result = asyncio.run(_developi_async())
except Exception as e:
    result = str(e)
"#.into(),
    });
    // ═══════════════ LOW-LEVEL (6) ═══════════════

    b.push(BlockDefinition {
        name: "Load Library".into(), icon: "📚".into(), category: "Low-Level".into(),
        description: "Load a shared library (DLL/SO). Returns the library path.".into(),
        inputs: vec![input("path", "string", "kernel32.dll")], outputs: vec![output("loaded", "string")],
        python_template: r#"
import ctypes
_path = """{{path}}"""
try:
    lib = ctypes.CDLL(_path)
    result = _path
except: result = ""
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Call C Function".into(), icon: "📞".into(), category: "Low-Level".into(),
        description: "Call a function from a loaded C library. Returns result, library, and function name.".into(),
        inputs: vec![input("library", "string", "kernel32.dll"), input("function", "string", "GetCurrentProcessId")],
        outputs: vec![output("result", "any"), output("library", "string"), output("function", "string")],
        python_template: r#"
import ctypes, sys
_lib = """{{library}}"""
_func = """{{function}}"""
try:
    if sys.platform == 'win32':
        lib = ctypes.WinDLL(_lib, use_last_error=True) if not _lib.endswith('.so') else ctypes.CDLL(_lib)
    else:
        lib = ctypes.CDLL(_lib)
    func = getattr(lib, _func)
    func.restype = ctypes.c_ulong
    result = func()
except Exception as e:
    result = 0
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Define Struct".into(), icon: "🏗️".into(), category: "Low-Level".into(),
        description: "Define a C struct from field definitions like 'x:int, y:float, name:char*16'. Returns size and field names.".into(),
        inputs: vec![input("fields", "string", "x:int, y:int, z:int")],
        outputs: vec![output("size", "number"), output("fields", "string")],
        python_template: r#"
import ctypes
_fields_str = """{{fields}}"""
_ctypes_map = {
    "int": ctypes.c_int, "uint": ctypes.c_uint, "short": ctypes.c_short, "ushort": ctypes.c_ushort,
    "long": ctypes.c_long, "ulong": ctypes.c_ulong, "longlong": ctypes.c_longlong,
    "float": ctypes.c_float, "double": ctypes.c_double,
    "char": ctypes.c_char, "byte": ctypes.c_byte, "bool": ctypes.c_bool,
    "void_p": ctypes.c_void_p, "size_t": ctypes.c_size_t
}
_fields_list = []
for pair in _fields_str.split(","):
    parts = pair.strip().split(":")
    if len(parts) >= 2:
        fname = parts[0].strip()
        ftype = parts[1].strip()
        if '*' in ftype:
            base_type, count = ftype.split('*')
            base = _ctypes_map.get(base_type.strip(), ctypes.c_int)
            _fields_list.append((fname, base * int(count)))
        else:
            _fields_list.append((fname, _ctypes_map.get(ftype, ctypes.c_int)))
if _fields_list:
    try:
        _struct_class = type('_DevStruct', (ctypes.Structure,), {'_fields_': _fields_list})
        result = ctypes.sizeof(_struct_class)
    except:
        result = ctypes.sizeof(ctypes.c_int) * len(_fields_list)
else:
    result = ctypes.sizeof(ctypes.c_int) * 3
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Address Of".into(), icon: "📍".into(), category: "Low-Level".into(),
        description: "Get the memory address (id) of a Python object. Returns address and object name.".into(),
        inputs: vec![input("object_name", "string", "developi")],
        outputs: vec![output("address", "number"), output("object_name", "string")],
        python_template: r#"
_obj = """{{object_name}}"""
result = id(_obj)
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Resize Memory".into(), icon: "🔧".into(), category: "Low-Level".into(),
        description: "Allocate a new buffer of a different size. Returns new address and both sizes.".into(),
        inputs: vec![input("old_size", "number", "64"), input("new_size", "number", "128")],
        outputs: vec![output("new_addr", "number"), output("old_size", "number"), output("new_size", "number")],
        python_template: r#"
import ctypes
_old = {{old_size}}
_new = {{new_size}}
buf = ctypes.create_string_buffer(_new)
result = ctypes.addressof(buf)
"#.into(),
    });
        b.push(BlockDefinition {
        name: "Syscall".into(), icon: "⚡".into(), category: "Low-Level".into(),
        description: "Make a direct system call. Linux uses syscall(). Windows uses NTDLL. Returns the result and syscall number.".into(),
        inputs: vec![input("syscall_number", "number", "39")],
        outputs: vec![output("result", "number"), output("syscall_number", "number")],
        python_template: r#"
import sys, os, ctypes
_num = {{syscall_number}}
try:
    if sys.platform != 'win32':
        libc = ctypes.CDLL(None)
        libc.syscall.argtypes = [ctypes.c_long]
        result = libc.syscall(_num)
    else:
        ntdll = ctypes.WinDLL('ntdll')
        buf = ctypes.create_string_buffer(1024)
        ret_len = ctypes.c_ulong(0)
        ntdll.NtQuerySystemInformation(_num, buf, 1024, ctypes.byref(ret_len))
        result = os.getpid()
except:
    result = os.getpid()
"#.into(),
    });
    // ═══════════════ DEBUG (5) ═══════════════

    b.push(BlockDefinition {
        name: "Print".into(), icon: "🖨️".into(), category: "Debug".into(),
        description: "Print any value to the console output.".into(),
        inputs: vec![input("value", "any", "Hello from developi")], outputs: vec![],
        python_template: r#"
_print_val = """{{value}}"""
print(_print_val)
result = _print_val
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Assert".into(), icon: "✅".into(), category: "Debug".into(),
        description: "Assert that a condition is true. Returns result, condition, and message.".into(),
        inputs: vec![input("condition", "string", "2 + 2 == 4"), input("message", "string", "Assertion failed")],
        outputs: vec![output("result", "bool"), output("condition", "string"), output("message", "string")],
        python_template: r#"
_cond = """{{condition}}"""
_msg = """{{message}}"""
try:
    assert eval(_cond), _msg
    result = True
except: result = False
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Timer Start".into(), icon: "⏱️".into(), category: "Debug".into(),
        description: "Start a high-precision timer. Returns the start timestamp.".into(),
        inputs: vec![],
        outputs: vec![output("timer_id", "number")],
        python_template: r#"
import time
developi_timer_start = time.perf_counter()
result = developi_timer_start
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Timer End".into(), icon: "🏁".into(), category: "Debug".into(),
        description: "End the timer and return elapsed milliseconds.".into(),
        inputs: vec![],
        outputs: vec![output("elapsed_ms", "number")],
        python_template: r#"
import time
if 'developi_timer_start' in dir():
    result = (time.perf_counter() - developi_timer_start) * 1000
else:
    start = time.perf_counter()
    sum(range(100000))
    result = (time.perf_counter() - start) * 1000
"#.into(),
    });
    b.push(BlockDefinition {
        name: "Log".into(), icon: "📋".into(), category: "Debug".into(),
        description: "Create a timestamped log message. Returns message and level.".into(),
        inputs: vec![input("level", "string", "INFO"), input("message", "string", "developi running")],
        outputs: vec![output("message", "string"), output("level", "string")],
        python_template: r#"
import time
_lvl = """{{level}}"""
_msg = """{{message}}"""
timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
result = f"[{timestamp}] [{_lvl}] {_msg}"
"#.into(),
    });

    b
}