use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::prelude::*;
use glam::Vec3;
use schiro_ecs::{components::Transform, systems::Time, World};
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

pub struct EditorApp {
    pub world: World,
    pub renderer: Option<Renderer>,
    pub asset_server: schiro_assets::AssetServer,
    pub input: InputSystem,
    pub physics: PhysicsWorld,
    pub project: Project,
    pub window: Option<Arc<Window>>,
    pub egui_ctx: egui::Context,
    pub egui_winit_state: Option<egui_winit::State>,
    pub viewport_panel: ViewportPanel,
    pub scene_entities: Vec<Entity>,
    pub selected_entity: Option<Entity>,
    pub entity_mesh_map: HashMap<Entity, usize>,
    pub gizmo_mesh_start: usize,
    pub gizmo_drag: Option<GizmoDrag>,
    pub current_tool: EditorTool,
    pub playing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy)]
pub struct GizmoDrag {
    pub axis: usize,
    pub entity: Entity,
}

impl EditorApp {
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
        };

        crate::theme::apply_dark_theme(&app.egui_ctx);
        app
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    pub fn render_frame(&mut self) {
        if self.playing { self.run_game_systems(); }

        let ctx = self.egui_ctx.clone();
        let full_output = {
            let (state, window) = match (self.egui_winit_state.as_mut(), self.window.as_ref()) {
                (Some(s), Some(w)) => (s, w),
                _ => return,
            };
            let raw = state.take_egui_input(window);
            ctx.run(raw, |ctx| self.build_editor_ui(ctx))
        };

        if let (Some(s), Some(w)) = (self.egui_winit_state.as_mut(), self.window.as_ref()) {
            s.handle_platform_output(w, full_output.platform_output.clone());
        }

        let vp_w = self.viewport_panel.rect.width() as u32;
        let vp_h = self.viewport_panel.rect.height() as u32;
        let aspect = if vp_h > 0 { vp_w as f32 / vp_h as f32 } else { 16.0 / 9.0 };

        gizmo::handle_viewport_input(self, aspect);

        let renderer = match self.renderer.as_mut() {
            Some(r) => r, None => return,
        };

        if vp_w > 0 && vp_h > 0 { renderer.resize_viewport(vp_w, vp_h); }
        {
            let mut query = self.world.query::<(Entity, &Transform, &schiro_ecs::components::MeshRenderer)>();
            for (e, t, _) in query.iter(&self.world) {
                if let Some(&idx) = self.entity_mesh_map.get(&e) {
                    renderer.update_mesh_transform(idx, &t.compute_matrix());
                }
            }
        }
        update_gizmo_transforms(renderer, &self.world, self.selected_entity, self.gizmo_mesh_start, self.current_tool);

        let camera = self.viewport_panel.camera.to_uniform(aspect);
        if let Err(wgpu::SurfaceError::OutOfMemory) = renderer.render(&ctx, full_output, &camera) {
            panic!("GPU out of memory");
        }
    }

    fn run_game_systems(&mut self) {
        let mut t = self.world.resource_mut::<Time>();
        t.update(0.016);
        let mut s = Schedule::default();
        s.add_systems(schiro_ecs::systems::rotate_entities);
        s.add_systems(schiro_ecs::systems::propagate_transforms);
        s.run(&mut self.world);
    }

    fn sync_ecs(&mut self, renderer: &mut Renderer) {
        let mut query = self.world.query::<(Entity, &Transform, &schiro_ecs::components::MeshRenderer)>();
        for (e, t, _) in query.iter(&self.world) {
            if let Some(&idx) = self.entity_mesh_map.get(&e) {
                renderer.update_mesh_transform(idx, &t.compute_matrix());
            }
        }
    }

    pub fn get_entity_name(&self, entity: Entity) -> String {
        self.world.get::<schiro_ecs::components::Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.index()))
    }

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
        let mut renderer = pollster::block_on(Renderer::new(Arc::clone(&window), (ws.width, ws.height)))
            .expect("failed to create renderer");

        init_scene(&mut self.world, &mut renderer, &self.asset_server,
            &mut self.scene_entities, &mut self.entity_mesh_map, &mut self.gizmo_mesh_start);

        self.egui_winit_state = Some(egui_winit::State::new(
            self.egui_ctx.clone(), egui::ViewportId::default(), &window, None, None, None));
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        self.input.process_window_event(&event);
        if let (Some(s), Some(w)) = (self.egui_winit_state.as_mut(), self.window.as_ref()) {
            let _ = s.on_window_event(w, &event);
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::Resized(ps) => {
                if let Some(r) = self.renderer.as_mut() { r.resize(ps.width, ps.height); }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = self.window.as_ref() { w.request_redraw(); }
    }
}
