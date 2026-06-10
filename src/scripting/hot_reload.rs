//! Hot-reload de scripts Lua — detecta alterações no disco e recarrega.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct WatchedScript {
    pub path: PathBuf,
    pub modified: SystemTime,
}

#[derive(Default)]
pub struct ScriptWatcher {
    scripts: HashMap<PathBuf, SystemTime>,
}

impl ScriptWatcher {
    pub fn watch(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                self.scripts.insert(path, mtime);
            }
        }
    }

    pub fn unwatch(&mut self, path: impl AsRef<Path>) {
        self.scripts.remove(path.as_ref());
    }

    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.scripts.keys()
    }

    /// Retorna caminhos cujo mtime mudou desde o último registro.
    pub fn poll_changed(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        for (path, stored) in self.scripts.iter_mut() {
            let Ok(meta) = std::fs::metadata(path) else { continue };
            let Ok(mtime) = meta.modified() else { continue };
            if mtime > *stored {
                *stored = mtime;
                changed.push(path.clone());
            }
        }
        changed
    }
}
