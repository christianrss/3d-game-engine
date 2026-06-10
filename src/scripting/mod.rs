//! Scripting Lua — linguagem de script da engine (estilo Unity).

pub mod hot_reload;
pub mod lua_runtime;

pub use hot_reload::ScriptWatcher;
pub use lua_runtime::{LuaRuntime, ScriptError};
