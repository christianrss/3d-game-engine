//! Runtime Lua 5.4 via mlua — API `engine.*` para scripts de cena.

use crate::scripting::hot_reload::ScriptWatcher;
use mlua::{Function, Lua, Result as LuaResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum ScriptError {
    Lua(mlua::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Lua(e) => write!(f, "Lua: {e}"),
            ScriptError::Io(e) => write!(f, "IO: {e}"),
        }
    }
}

impl From<mlua::Error> for ScriptError {
    fn from(e: mlua::Error) -> Self {
        ScriptError::Lua(e)
    }
}

impl From<std::io::Error> for ScriptError {
    fn from(e: std::io::Error) -> Self {
        ScriptError::Io(e)
    }
}

/// Estado compartilhado entre Rust e scripts Lua.
#[derive(Default, Clone)]
pub struct ScriptWorld {
    pub time: f32,
    pub playing: bool,
    pub messages: Arc<Mutex<Vec<String>>>,
}

pub struct LuaRuntime {
    lua: Lua,
    world: ScriptWorld,
    watcher: ScriptWatcher,
}

impl LuaRuntime {
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        let world = ScriptWorld {
            messages: Arc::new(Mutex::new(Vec::new())),
            ..Default::default()
        };
        let mut rt = Self {
            lua,
            world,
            watcher: ScriptWatcher::default(),
        };
        rt.register_api()?;
        Ok(rt)
    }

    fn register_api(&mut self) -> LuaResult<()> {
        let world = self.world.clone();
        let lua = &self.lua;

        let engine = lua.create_table()?;

        let msgs = world.messages.clone();
        engine.set(
            "log",
            lua.create_function(move |_, msg: String| {
                log::info!("[Lua] {msg}");
                if let Ok(mut m) = msgs.lock() {
                    m.push(msg);
                }
                Ok(())
            })?,
        )?;

        let w_time = world.clone();
        engine.set(
            "time",
            lua.create_function(move |_, ()| Ok(w_time.time))?,
        )?;

        let w_play = world.clone();
        engine.set(
            "is_playing",
            lua.create_function(move |_, ()| Ok(w_play.playing))?,
        )?;

        engine.set(
            "vec3",
            lua.create_function(|lua, (x, y, z): (f32, f32, f32)| {
                let t = lua.create_table()?;
                t.set("x", x)?;
                t.set("y", y)?;
                t.set("z", z)?;
                Ok(t)
            })?,
        )?;

        lua.globals().set("engine", engine)?;
        Ok(())
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), ScriptError> {
        let path = path.as_ref();
        let src = std::fs::read_to_string(path)?;
        self.lua
            .load(&src)
            .set_name(path.to_string_lossy())
            .exec()?;
        self.watcher.watch(path);
        Ok(())
    }

    /// Recarrega todos os scripts monitorados que mudaram no disco.
    pub fn hot_reload_poll(&mut self) -> Result<Vec<PathBuf>, ScriptError> {
        let changed = self.watcher.poll_changed();
        if changed.is_empty() {
            return Ok(vec![]);
        }
        let playing = self.world.playing;
        if playing {
            let _ = self.call_stop();
        }
        for path in &changed {
            let src = std::fs::read_to_string(path)?;
            self.lua
                .load(&src)
                .set_name(path.to_string_lossy())
                .exec()?;
            log::info!("[HotReload] Script recarregado: {}", path.display());
        }
        if playing {
            let _ = self.call_start();
        }
        Ok(changed)
    }

    pub fn watch_script(&mut self, path: impl AsRef<Path>) {
        self.watcher.watch(path);
    }

    pub fn reload_file(&mut self, path: impl AsRef<Path>) -> Result<(), ScriptError> {
        self.load_file(path)
    }

    pub fn load_string(&self, name: &str, src: &str) -> Result<(), ScriptError> {
        self.lua.load(src).set_name(name).exec()?;
        Ok(())
    }

    pub fn call_start(&self) -> Result<(), ScriptError> {
        self.call_fn("on_start")
    }

    pub fn call_update(&self, dt: f32) -> Result<(), ScriptError> {
        let globals = self.lua.globals();
        if let Ok(f) = globals.get::<Function>("update") {
            f.call::<()>(dt)?;
        }
        Ok(())
    }

    pub fn call_stop(&self) -> Result<(), ScriptError> {
        self.call_fn("on_stop")
    }

    fn call_fn(&self, name: &str) -> Result<(), ScriptError> {
        let globals = self.lua.globals();
        if let Ok(f) = globals.get::<Function>(name) {
            f.call::<()>(())?;
        }
        Ok(())
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.world.playing = playing;
    }

    pub fn set_time(&mut self, t: f32) {
        self.world.time = t;
    }

    pub fn drain_messages(&self) -> Vec<String> {
        self.world
            .messages
            .lock()
            .map(|mut m| std::mem::take(&mut *m))
            .unwrap_or_default()
    }
}
