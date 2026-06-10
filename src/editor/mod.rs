//! Engine Studio — editor de cenas estilo Unity.

pub mod gizmo;
pub mod scene_doc;
pub mod studio;
pub mod ui;

pub use gizmo::{GizmoAxis, GizmoMode, TransformGizmo};
pub use scene_doc::{SceneDocument, SceneEntity, SceneEntityKind};
pub use studio::EngineStudio;
pub use ui::EditorUi;
