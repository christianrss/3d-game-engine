//! UI egui — Hierarchy, Inspector, Toolbar e Console.

use crate::editor::gizmo::{GizmoMode, TransformGizmo};
use crate::editor::scene_doc::{SceneDocument, SceneEntityKind};
use egui::{Color32, Context, RichText, ScrollArea};
use std::path::PathBuf;
use std::sync::Arc;

pub struct EditorUi {
    pub egui_ctx: egui::Context,
    pub winit_state: egui_winit::State,
    pub glow_painter: Option<egui_glow::Painter>,
    pub console_lines: Vec<String>,
    pub script_path_edit: String,
    pub viewport_focused: bool,
    pub wants_pointer: bool,
}

impl EditorUi {
    pub fn new(window: &winit::window::Window) -> Self {
        let egui_ctx = egui::Context::default();
        egui_ctx.set_visuals(egui::Visuals::dark());
        let winit_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        Self {
            egui_ctx,
            winit_state,
            glow_painter: None,
            console_lines: Vec::new(),
            script_path_edit: String::new(),
            viewport_focused: true,
            wants_pointer: false,
        }
    }

    #[cfg(not(feature = "opengl"))]
    pub fn init_glow(&mut self, _gl: ()) {
        // Engine Studio usa egui_glow (requer OpenGL).
    }

    #[cfg(feature = "opengl")]
    pub fn init_glow(&mut self, gl: Arc<glow::Context>) {
        if self.glow_painter.is_some() {
            return;
        }
        self.glow_painter = Some(
            egui_glow::Painter::new(gl, "", None, false).expect("egui_glow painter"),
        );
    }

    pub fn on_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.winit_state.on_window_event(window, event)
    }

    pub fn push_log(&mut self, line: impl Into<String>) {
        self.console_lines.push(line.into());
        if self.console_lines.len() > 80 {
            self.console_lines.remove(0);
        }
    }

    /// Desenha painéis; retorna ações e output egui para paint posterior.
    pub fn draw_panels(
        &mut self,
        window: &winit::window::Window,
        scene: &mut SceneDocument,
        selected: &mut usize,
        playing: bool,
        gizmo: &TransformGizmo,
        scene_path: &PathBuf,
        dt: f32,
    ) -> (StudioActions, Option<egui::FullOutput>) {
        let raw_input = self.winit_state.take_egui_input(window);
        let mut actions = StudioActions::default();

        self.egui_ctx.request_repaint();

        let mut full_output = self.egui_ctx.run(raw_input, |ctx| {
            Self::toolbar(ctx, playing, &mut actions);
            Self::hierarchy_panel(ctx, scene, selected, &mut actions);
            Self::inspector_panel(ctx, scene, *selected, gizmo, &mut self.script_path_edit, &mut actions);
            Self::console_panel(ctx, &self.console_lines);
            Self::status_bar(ctx, playing, scene_path, dt);
        });

        self.winit_state
            .handle_platform_output(window, std::mem::take(&mut full_output.platform_output));

        self.wants_pointer = self.egui_ctx.is_pointer_over_area();
        self.viewport_focused = !self.wants_pointer;

        (actions, Some(full_output))
    }

    #[cfg(not(feature = "opengl"))]
    pub fn paint(&mut self, _full_output: egui::FullOutput, _width: u32, _height: u32) {}

    #[cfg(feature = "opengl")]
    pub fn paint(&mut self, full_output: egui::FullOutput, width: u32, height: u32) {
        let Some(painter) = self.glow_painter.as_mut() else {
            return;
        };
        let clipped = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }
        painter.paint_and_update_textures(
            [width, height],
            full_output.pixels_per_point,
            &clipped,
            &full_output.textures_delta,
        );
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    fn toolbar(ctx: &Context, playing: bool, actions: &mut StudioActions) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Engine Studio").strong());
                ui.separator();
                if ui.button("▶ Play (F5)").clicked() {
                    actions.play = true;
                }
                if ui.button("■ Stop (F6)").clicked() {
                    actions.stop = true;
                }
                ui.separator();
                if ui.button("💾 Salvar").clicked() {
                    actions.save = true;
                }
                if ui.button("+ Cubo").clicked() {
                    actions.add_cube = true;
                }
                if ui.button("+ Alvo").clicked() {
                    actions.add_target = true;
                }
                ui.separator();
                let mode = if playing {
                    RichText::new("PLAY").color(Color32::LIGHT_GREEN)
                } else {
                    RichText::new("EDIT").color(Color32::LIGHT_BLUE)
                };
                ui.label(mode);
            });
        });
    }

    fn hierarchy_panel(
        ctx: &Context,
        scene: &SceneDocument,
        selected: &mut usize,
        actions: &mut StudioActions,
    ) {
        egui::SidePanel::left("hierarchy")
            .default_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, ent) in scene.entities.iter().enumerate() {
                        let icon = match ent.kind {
                            SceneEntityKind::Terrain => "🌍",
                            SceneEntityKind::Light => "☀",
                            SceneEntityKind::Target => "🎯",
                            SceneEntityKind::Cube => "📦",
                            SceneEntityKind::Sphere => "⚪",
                            SceneEntityKind::Camera => "📷",
                            SceneEntityKind::Empty => "∅",
                        };
                        let label = format!("{icon} {}", ent.name);
                        let sel = *selected == i;
                        if ui.selectable_label(sel, label).clicked() {
                            *selected = i;
                        }
                    }
                });
                ui.separator();
                if ui.button("Remover selecionado").clicked() {
                    actions.remove_selected = true;
                }
            });
    }

    fn inspector_panel(
        ctx: &Context,
        scene: &mut SceneDocument,
        selected: usize,
        gizmo: &TransformGizmo,
        script_edit: &mut String,
        actions: &mut StudioActions,
    ) {
        egui::SidePanel::right("inspector")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();
                let Some(ent) = scene.entities.get_mut(selected) else {
                    ui.label("Nenhuma entidade selecionada.");
                    return;
                };

                if script_edit.is_empty() {
                    *script_edit = ent.script.clone().unwrap_or_default();
                }

                ui.label("Nome");
                ui.text_edit_singleline(&mut ent.name);

                ui.label("Tipo");
                let kind_str = format!("{:?}", ent.kind);
                ui.label(&kind_str);

                ui.checkbox(&mut ent.enabled, "Ativo");

                ui.separator();
                ui.label(RichText::new("Transform").strong());

                let mut pos = ent.position;
                ui.horizontal(|ui| {
                    ui.label("Pos");
                    ui.add(egui::DragValue::new(&mut pos[0]).speed(0.1).prefix("X "));
                    ui.add(egui::DragValue::new(&mut pos[1]).speed(0.1).prefix("Y "));
                    ui.add(egui::DragValue::new(&mut pos[2]).speed(0.1).prefix("Z "));
                });
                ent.position = pos;

                ui.add(
                    egui::Slider::new(&mut ent.rotation_y, -3.14..=3.14).text("Rotação Y"),
                );
                ui.add(egui::Slider::new(&mut ent.scale, 0.1..=5.0).text("Escala"));

                ui.separator();
                ui.label(RichText::new("Gizmo").strong());
                let mode_label = match gizmo.mode {
                    GizmoMode::Translate => "Mover (W)",
                    GizmoMode::Rotate => "Rotacionar (E)",
                };
                ui.label(mode_label);
                ui.label("Arraste os eixos coloridos no viewport.");

                ui.separator();
                ui.label(RichText::new("Script Lua").strong());
                ui.text_edit_singleline(script_edit);
                if ui.button("Aplicar script").clicked() {
                    if script_edit.is_empty() {
                        ent.script = None;
                    } else {
                        ent.script = Some(script_edit.clone());
                    }
                    actions.script_changed = true;
                }
                if ui.button("Recarregar script").clicked() {
                    actions.reload_script = true;
                }

                ui.separator();
                if ui.button("Aplicar ao mundo").clicked() {
                    actions.rebuild_world = true;
                }
            });
    }

    fn console_panel(ctx: &Context, lines: &[String]) {
        egui::TopBottomPanel::bottom("console")
            .default_height(120.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Console");
                ScrollArea::vertical().show(ui, |ui| {
                    for line in lines {
                        ui.monospace(line);
                    }
                });
            });
    }

    fn status_bar(ctx: &Context, playing: bool, path: &PathBuf, dt: f32) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} | {:.1} ms | {:?}",
                    if playing { "▶ PLAY" } else { "✎ EDIT" },
                    dt * 1000.0,
                    path
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("W Mover | E Rotacionar | Botão direito orbitar");
                });
            });
        });
    }
}

#[derive(Default)]
pub struct StudioActions {
    pub play: bool,
    pub stop: bool,
    pub save: bool,
    pub add_cube: bool,
    pub add_target: bool,
    pub remove_selected: bool,
    pub rebuild_world: bool,
    pub script_changed: bool,
    pub reload_script: bool,
}
