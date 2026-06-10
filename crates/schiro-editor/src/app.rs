//! Editor application core.
//!
//! Holds every long-lived piece of state required by the editor: the
//! ECS [`World`], the wgpu [`Renderer`], the asset server, the input
//! and physics systems, the viewport, the gizmo state and the panel
//! focused widget. The struct implements [`winit::application::ApplicationHandler`]
//! so that winit can drive the per-frame event flow.

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::prelude::*;
use glam::Vec3;
use schiro_ecs::components::Transform;
use schiro_ecs::systems::Time;
use schiro_ecs::World;
use schiro_input::InputSystem;
use schiro_physics::PhysicsWorld;
use schiro_render::Renderer;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::editor::gizmo;
use crate::editor::scene::{init_scene, update_gizmo_transforms};
use crate::project::Project;
use crate::viewport::ViewportPanel;

/// Root editor application.
pub struct EditorApp {
    /// ECS world shared by the editor and runtime systems.
    pub world: World,
    /// Optional wgpu renderer, created when the window is ready.
    pub renderer: Option<Renderer>,
    /// Asset server used to deduplicate loads.
    pub asset_server: schiro_assets::AssetServer,
    /// Input system that tracks keyboard, mouse and gamepad state.
    pub input: InputSystem,
    /// 3D physics world wrapper around Rapier.
    pub physics: PhysicsWorld,
    /// Current project metadata.
    pub project: Project,
    /// Application window, once it has been created.
    pub window: Option<Arc<Window>>,
    /// egui context used to build the editor UI.
    pub egui_ctx: egui::Context,
    /// egui-winit integration state.
    pub egui_winit_state: Option<egui_winit::State>,
    /// 3D viewport widget embedded in the editor.
    pub viewport_panel: ViewportPanel,
    /// Entities currently part of the scene.
    pub scene_entities: Vec<Entity>,
    /// Currently selected entity, if any.
    pub selected_entity: Option<Entity>,
    /// Maps every scene entity to the index of its GPU mesh.
    pub entity_mesh_map: HashMap<Entity, usize>,
    /// Index of the first gizmo mesh in [`EditorApp::renderer`].
    pub gizmo_mesh_start: usize,
    /// Active gizmo drag, if any.
    pub gizmo_drag: Option<GizmoDrag>,
    /// Current gizmo tool.
    pub current_tool: EditorTool,
    /// `true` when the runtime is in play mode.
    pub playing: bool,
    /// Undo command stack.
    pub undo_stack: Vec<crate::command::Command>,
    /// Redo command stack.
    pub redo_stack: Vec<crate::command::Command>,
}

/// Gizmo tool currently selected in the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    /// Translate the selected entity along an axis.
    Translate,
    /// Rotate the selected entity around an axis.
    Rotate,
    /// Scale the selected entity along an axis.
    Scale,
}

/// State of an in-progress gizmo drag.
#[derive(Debug, Clone, Copy)]
pub struct GizmoDrag {
    /// Index of the axis being dragged (`0` = X, `1` = Y, `2` = Z).
    pub axis: usize,
    /// Entity being transformed.
    pub entity: Entity,
    /// Transform snapshot taken before the drag began, used for undo.
    pub start_transform: Transform,
}

impl EditorApp {
    /// Builds a new editor application. The wgpu renderer is not
    /// created yet; it is built inside [`ApplicationHandler::resumed`]
    /// when the window is ready.
    pub fn new() -> Self {
        let mut world = World::new();
        schiro_ecs::init(&mut world);
        info!("editor engine initialized");

        let app = Self {
            world,
            renderer: None,
            asset_server: schiro_assets::AssetServer::new(),
            input: InputSystem::new(),
            physics: PhysicsWorld::new(),
            project: Project::new("Untitled"),
            window: None,
            egui_ctx: egui::Context::default(),
            egui_winit_state: None,
            viewport_panel: ViewportPanel::new(),
            scene_entities: Vec::new(),
            selected_entity: None,
            entity_mesh_map: HashMap::new(),
            gizmo_mesh_start: 0,
            gizmo_drag: None,
            current_tool: EditorTool::Translate,
            playing: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        crate::theme::apply_dark_theme(&app.egui_ctx);
        app
    }

    /// Runs the winit event loop. Blocks until the window is closed.
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    /// Builds a frame: updates ECS state, runs the egui UI, syncs
    /// transforms to the renderer and submits a render command buffer.
    pub fn render_frame(&mut self) {
        if self.playing {
            self.run_game_systems();
        }

        let ctx = self.egui_ctx.clone();
        let full_output = {
            let (state, window) = match (self.egui_winit_state.as_mut(), self.window.as_ref()) {
                (Some(s), Some(w)) => (s, w),
                _ => return,
            };
            let raw = state.take_egui_input(window);
            ctx.run(raw, |ctx| {
                // Global keyboard shortcuts.
                ctx.input_mut(|i| {
                    if i.modifiers.ctrl {
                        if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                            self.undo();
                        }
                        if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
                            self.redo();
                        }
                        if i.consume_key(egui::Modifiers::CTRL, egui::Key::D) {
                            self.duplicate_entity();
                        }
                    }
                });
                self.build_editor_ui(ctx)
            })
        };

        if let (Some(s), Some(w)) = (self.egui_winit_state.as_mut(), self.window.as_ref()) {
            s.handle_platform_output(w, full_output.platform_output.clone());
        }

        let vp_w = self.viewport_panel.rect.width() as u32;
        let vp_h = self.viewport_panel.rect.height() as u32;
        let aspect = if vp_h > 0 { vp_w as f32 / vp_h as f32 } else { 16.0 / 9.0 };

        gizmo::handle_viewport_input(self, aspect);

        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };

        if vp_w > 0 && vp_h > 0 {
            renderer.resize_viewport(vp_w, vp_h);
        }
        {
            let mut query =
                self.world.query::<(Entity, &Transform, &schiro_ecs::components::MeshRenderer)>();
            for (e, t, _) in query.iter(&self.world) {
                if let Some(&idx) = self.entity_mesh_map.get(&e) {
                    renderer.update_mesh_transform(idx, &t.compute_matrix());
                }
            }
        }
        update_gizmo_transforms(
            renderer,
            &self.world,
            self.selected_entity,
            self.gizmo_mesh_start,
            self.current_tool,
        );

        let camera = self.viewport_panel.camera.to_uniform(aspect);
        if let Err(wgpu::SurfaceError::OutOfMemory) = renderer.render(&ctx, full_output, &camera) {
            panic!("GPU out of memory");
        }
    }

    /// Runs the systems that should execute while the editor is in
    /// play mode.
    fn run_game_systems(&mut self) {
        let mut t = self.world.resource_mut::<Time>();
        t.update(0.016);
        let mut s = Schedule::default();
        s.add_systems(schiro_ecs::systems::rotate_entities);
        s.add_systems(schiro_ecs::systems::propagate_transforms);
        s.run(&mut self.world);
    }

    /// Pushes every entity's [`Transform`] to its matching GPU mesh.
    fn sync_ecs(&mut self, renderer: &mut Renderer) {
        let mut query =
            self.world.query::<(Entity, &Transform, &schiro_ecs::components::MeshRenderer)>();
        for (e, t, _) in query.iter(&self.world) {
            if let Some(&idx) = self.entity_mesh_map.get(&e) {
                renderer.update_mesh_transform(idx, &t.compute_matrix());
            }
        }
    }

    /// Pushes scene_entities + mesh_map for an entity that already has
    /// a GPU mesh uploaded.
    pub fn register_scene_entity(&mut self, entity: Entity, mesh_index: usize) {
        self.scene_entities.push(entity);
        self.entity_mesh_map.insert(entity, mesh_index);
        self.selected_entity = Some(entity);
    }

    /// Spawns a procedural mesh entity with an optional Rotator.
    pub fn add_mesh_entity(
        &mut self,
        name: impl Into<String>,
        mesh: &schiro_render::Mesh,
        translation: glam::Vec3,
        rotator: Option<glam::Vec3>,
    ) {
        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::ONE,
            glam::Quat::IDENTITY,
            translation,
        );
        renderer.add_mesh(mesh, &transform);
        let mi = renderer.mesh_count() - 1;

        let mut cmd = self.world.spawn((
            schiro_ecs::components::Name(name.into()),
            Transform { translation, ..Default::default() },
            schiro_ecs::components::GlobalTransform::default(),
            schiro_ecs::components::MeshRenderer { mesh_handle: Some(mi), visible: true },
        ));
        if let Some(speed) = rotator {
            cmd.insert(schiro_ecs::components::Rotator { speed });
        }
        let entity = cmd.id();
        self.register_scene_entity(entity, mi);
    }

    /// Spawns an empty entity (Transform + Name, no mesh, no Rotator).
    pub fn add_empty(&mut self, name: impl Into<String>, translation: glam::Vec3) {
        let entity = self
            .world
            .spawn((
                schiro_ecs::components::Name(name.into()),
                Transform { translation, ..Default::default() },
                schiro_ecs::components::GlobalTransform::default(),
            ))
            .id();
        self.scene_entities.push(entity);
        self.selected_entity = Some(entity);
    }

    /// Returns the human readable name of an entity, falling back to
    /// `Entity <id>` for unnamed entities.
    pub fn get_entity_name(&self, entity: Entity) -> String {
        self.world
            .get::<schiro_ecs::components::Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.index()))
    }

    /// Returns the current local [`Transform`] of an entity, or the
    /// default transform if the entity has none.
    pub fn get_entity_transform(&self, entity: Entity) -> Transform {
        self.world.get::<Transform>(entity).copied().unwrap_or_default()
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("SchiroEngine Editor")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .with_maximized(true);
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let ws = window.inner_size();
        let mut renderer =
            pollster::block_on(Renderer::new(Arc::clone(&window), (ws.width, ws.height)))
                .expect("failed to create renderer");

        init_scene(
            &mut self.world,
            &mut renderer,
            &self.asset_server,
            &mut self.scene_entities,
            &mut self.entity_mesh_map,
            &mut self.gizmo_mesh_start,
        );

        self.egui_winit_state = Some(egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::default(),
            &window,
            None,
            None,
            None,
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        el: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.input.process_window_event(&event);
        if let (Some(s), Some(w)) = (self.egui_winit_state.as_mut(), self.window.as_ref()) {
            let _ = s.on_window_event(w, &event);
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::Resized(ps) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(ps.width, ps.height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
    }
}
