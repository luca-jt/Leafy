use crate::ecs::entity_manager::EntityManager;
use crate::internal_prelude::*;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::data::*;
use crate::rendering::instance_renderer::InstanceRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::{ShaderCatalog, ShaderType};
use crate::rendering::sprite_renderer::{SpriteGrid, SpriteRenderer};
use crate::systems::event_system::events::user_space::CamPositionChange;
use crate::utils::constants::bits::user_level::INVISIBLE;
use std::cmp::Ordering;

/// The system responsible for automated rendering of all entities.
pub struct RenderingSystem {
    point_lights: AHashMap<EntityID, PointLightRenderingInfo>,
    directional_lights: Vec<(EntityID, ShadowMap)>, // Vec and linear traversal is fine because we dont have that many lights
    renderers: Vec<RendererType>,
    sprite_renderer: SpriteRenderer,
    perspective_camera: PerspectiveCamera,
    ortho_camera: OrthoCamera,
    shader_catalog: ShaderCatalog,
    clear_color: Color32,
    render_distance: Option<f32>,
    shadow_resolution: ShadowResolution,
    current_cam_config: (Vec3, Vec3, Vec3),
    ambient_light: (Color32, f32),
    skybox: Option<Skybox>,
    screen_texture: ScreenTexture,
    samples: GLsizei,
    tmp_storage: TempRenderStorage,
}

impl RenderingSystem {
    /// creates a new rendering system with initial cam data
    pub(crate) fn new(win_w: u32, win_h: u32) -> Self {
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Disable(gl::MULTISAMPLE);
        }
        let samples = 4;

        Self {
            point_lights: AHashMap::new(),
            directional_lights: Vec::with_capacity(MAX_DIR_LIGHT_MAPS),
            renderers: Vec::new(),
            sprite_renderer: SpriteRenderer::new(),
            perspective_camera: PerspectiveCamera::new(),
            ortho_camera: OrthoCamera::new(-1.0, 1.0),
            shader_catalog: ShaderCatalog::new(),
            clear_color: Color32::WHITE,
            render_distance: None,
            shadow_resolution: ShadowResolution::Normal,
            current_cam_config: (-Z_AXIS, Z_AXIS, Y_AXIS),
            ambient_light: (Color32::WHITE, 0.2),
            skybox: None,
            screen_texture: ScreenTexture::new(win_w as GLsizei, win_h as GLsizei, false, samples),
            samples,
            tmp_storage: TempRenderStorage::default(),
        }
    }

    /// render all entities
    pub(crate) fn render(&mut self, entity_manager: &EntityManager) {
        self.clear_gl_screen();
        self.update_lights(entity_manager);
        self.update_uniform_buffers();
        self.reset_renderer_usage();
        self.add_entity_data(entity_manager);
        self.confirm_data();
        self.render_shadows();
        self.bind_screen_texture();
        self.render_geometry();
        self.render_transparent();
        self.reset_renderers();
        self.render_skybox();
        self.render_screen_texture();
        self.cleanup_renderers();
        self.render_sprites(entity_manager);
    }

    /// adds and removes light sources according to entity data
    fn update_lights(&mut self, entity_manager: &EntityManager) {
        self.tmp_storage.clear_light_storage();

        //
        // directional lights
        //
        self.tmp_storage.dir_lights.extend(
            unsafe {
                entity_manager.query3::<&Position, &DirectionalLight, &EntityID>((None, None))
            }
            .map(|(p, l, e)| (*p, *l, *e)),
        );
        // remove deleted shadow maps
        self.directional_lights.retain(|src| {
            self.tmp_storage
                .dir_lights
                .iter()
                .any(|(_, _, id)| *id == src.0)
        });
        // update positions of existing ones
        for (entity, map) in self.directional_lights.iter_mut() {
            let (correct_pos, correct_light, _) = self
                .tmp_storage
                .dir_lights
                .iter()
                .find(|(_, _, id)| id == entity)
                .unwrap();
            if map.light_pos != *correct_pos.data()
                || map.light.direction != correct_light.direction
            {
                map.update_light(correct_pos.data(), correct_light);
            }
        }
        // add new light sources
        for (pos, src, entity) in self.tmp_storage.dir_lights.iter() {
            if !self.directional_lights.iter().any(|(id, _)| entity == id) {
                if self.directional_lights.len() == MAX_DIR_LIGHT_MAPS {
                    panic!(
                        "no more directional light source slots available (max is {})",
                        MAX_DIR_LIGHT_MAPS
                    );
                }
                self.directional_lights.push((
                    *entity,
                    ShadowMap::new(self.shadow_resolution.map_res(), *pos.data(), src),
                ));
            }
        }

        //
        // point lights
        //
        self.tmp_storage.p_lights.extend(
            unsafe { entity_manager.query3::<&Position, &PointLight, &EntityID>((None, None)) }
                .map(|(p, l, e)| (*p, *l, *e)),
        );

        // remove lights that dont exist any more
        self.point_lights.retain(|e_id, _| {
            self.tmp_storage
                .p_lights
                .iter()
                .any(|(_, _, id)| *id == *e_id)
        });

        // update data of existing ones
        for (entity, light_render_info) in self.point_lights.iter_mut() {
            let (correct_pos, correct_light, _) = self
                .tmp_storage
                .p_lights
                .iter()
                .find(|(_, _, id)| id == entity)
                .unwrap();

            if light_render_info.light_pos != *correct_pos.data() {
                if let Some(map) = light_render_info.shadow_map.as_mut() {
                    map.update_light(correct_pos.data());
                }
                light_render_info.light_pos = *correct_pos.data();
                light_render_info.light = *correct_light;
            }
        }
        // add new light sources and detect changes in shadow maps
        for (pos, src, entity) in self.tmp_storage.p_lights.iter() {
            if self.point_lights.keys().any(|id| id == entity) {
                // update the shadow map
                if !src.has_shadows && self.point_lights.get(entity).unwrap().shadow_map.is_some() {
                    self.point_lights.get_mut(entity).unwrap().shadow_map = None;
                } else if src.has_shadows
                    && self.point_lights.get(entity).unwrap().shadow_map.is_none()
                {
                    self.point_lights.get_mut(entity).unwrap().shadow_map = Some(
                        CubeShadowMap::new(self.shadow_resolution.map_res(), *pos.data()),
                    );
                }
            } else {
                // create a new shadow map
                if self.point_lights.len() == MAX_POINT_LIGHT_COUNT {
                    panic!(
                        "no more point light source slots available (max is {})",
                        MAX_POINT_LIGHT_COUNT
                    );
                }
                if src.has_shadows {
                    if self
                        .point_lights
                        .values()
                        .filter(|render_info| render_info.light.has_shadows)
                        .count()
                        == MAX_POINT_LIGHT_MAPS
                    {
                        panic!(
                            "no more point light source slots with shadow maps available (max is {})",
                            MAX_POINT_LIGHT_MAPS
                        );
                    }
                    self.point_lights.insert(
                        *entity,
                        PointLightRenderingInfo {
                            light_pos: *pos.data(),
                            light: *src,
                            shadow_map: Some(CubeShadowMap::new(
                                self.shadow_resolution.map_res(),
                                *pos.data(),
                            )),
                        },
                    );
                } else {
                    self.point_lights.insert(
                        *entity,
                        PointLightRenderingInfo {
                            light_pos: *pos.data(),
                            light: *src,
                            shadow_map: None,
                        },
                    );
                }
            }
        }
    }

    /// add entity data to the renderers
    fn add_entity_data(&mut self, entity_manager: &EntityManager) {
        let (render_dist, cam_pos) = (self.render_distance, self.current_cam_config.0);
        for (position, renderable, scale, orientation, rb, shader_type, lod) in unsafe {
            entity_manager
                .query9::<&Position, &Renderable, Option<&EntityFlags>, Option<&Scale>, Option<&Orientation>, Option<&RigidBody>, Option<&DirectionalLight>, Option<&PointLight>, Option<&LOD>>((None, None))
        }
            .filter(|(_, _, f_opt, ..)| f_opt.is_none_or(|flags| !flags.get_bit(INVISIBLE)))
            .filter(|(pos, ..)| render_dist.is_none_or(|dist| (pos.data() - cam_pos).norm() <= dist))
            .map(|(p, rndrbl, _, s, o, rb, dir_light, p_light, lod)| (p, rndrbl, s, o, rb, dir_light.map_or(p_light.map_or(ShaderType::Basic, |_| ShaderType::Passthrough), |_| ShaderType::Passthrough), lod.copied().unwrap_or_default()))
        {
            let trafo = calc_model_matrix(
                position,
                scale.unwrap_or(&Scale::default()),
                orientation.unwrap_or(&Orientation::default()),
                &rb.copied().unwrap_or_default().center_of_mass,
            );

            let render_data = RenderData {
                spec: RenderSpec {
                    mesh_type: renderable.mesh_type,
                    shader_type,
                    lod,
                },
                trafo: &trafo,
                m_attr: &renderable.mesh_attribute,
                mesh: entity_manager.mesh_from_handle(renderable.mesh_type.mesh_handle(), lod).unwrap(),
                tex_map: &entity_manager.texture_map,
                transparent: match &renderable.mesh_attribute {
                    MeshAttribute::Colored(color) => color.a < 255,
                    MeshAttribute::Textured(texture) => texture.is_transparent,
                }
            };

            let is_added = self.try_add_data(&render_data);
            // add new renderer if needed
            if !is_added {
                self.add_new_renderer(&render_data);
            }
        }
    }

    /// confirms all of the added data in the renderers
    fn confirm_data(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                RendererType::Batch { renderer, .. } => {
                    renderer.confirm_data();
                }
                RendererType::Instance { renderer, .. } => {
                    renderer.confirm_positions();
                }
            }
        }
    }

    /// binds the screen texture frame buffer for rendering
    fn bind_screen_texture(&mut self) {
        self.screen_texture.bind();
        self.clear_gl_screen();
    }

    /// render all sprite entities
    fn render_sprites(&mut self, entity_manager: &EntityManager) {
        self.sprite_renderer.add_data(entity_manager);
        self.shader_catalog.sprite.use_program();
        self.sprite_renderer.render();
        self.sprite_renderer.reset();
    }

    /// renders the screen texture to the default OpenGL frame buffer
    fn render_screen_texture(&self) {
        self.screen_texture.unbind();
        self.shader_catalog.screen.use_program();
        self.screen_texture.render();
    }

    /// renders the skybox if present
    fn render_skybox(&self) {
        if let Some(skybox) = self.skybox.as_ref() {
            self.shader_catalog.skybox.use_program();
            skybox.render();
        }
    }

    /// renders all the transparent fragments in the scene
    fn render_transparent(&self) {
        unsafe {
            gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
        }
        // @copypasta from render_geometry()
        let dir_shadow_maps = self.directional_lights.iter().map(|(_, map)| map);

        let cube_shadow_maps = self
            .point_lights
            .values()
            .filter_map(|info| info.shadow_map.as_ref());

        let mut current_shader = None;
        for renderer_type in self.renderers.iter().filter(|r| r.transparent()) {
            let new_shader = renderer_type.required_shader();
            if current_shader
                .map(|shader_spec| shader_spec != new_shader)
                .unwrap_or(true)
            {
                self.shader_catalog.use_shader(new_shader);
                current_shader = Some(new_shader);
            }
            match renderer_type {
                RendererType::Batch { spec, renderer, .. } => {
                    renderer.flush(
                        dir_shadow_maps.clone(),
                        cube_shadow_maps.clone(),
                        spec.shader_type,
                        true,
                    );
                }
                RendererType::Instance { spec, renderer, .. } => {
                    renderer.draw_all(
                        dir_shadow_maps.clone(),
                        cube_shadow_maps.clone(),
                        spec.shader_type,
                        true,
                    );
                }
            }
        }
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::DepthMask(gl::TRUE);
        }
    }

    /// render all the geometry data stored in the renderers
    fn render_geometry(&self) {
        let dir_shadow_maps = self.directional_lights.iter().map(|(_, map)| map);

        let cube_shadow_maps = self
            .point_lights
            .values()
            .filter_map(|info| info.shadow_map.as_ref());

        let mut current_shader = None;
        for renderer_type in self.renderers.iter() {
            let new_shader = renderer_type.required_shader();
            if current_shader
                .map(|shader_spec| shader_spec != new_shader)
                .unwrap_or(true)
            {
                self.shader_catalog.use_shader(new_shader);
                current_shader = Some(new_shader);
            }
            match renderer_type {
                RendererType::Batch { spec, renderer, .. } => {
                    renderer.flush(
                        dir_shadow_maps.clone(),
                        cube_shadow_maps.clone(),
                        spec.shader_type,
                        false,
                    );
                }
                RendererType::Instance { spec, renderer, .. } => {
                    renderer.draw_all(
                        dir_shadow_maps.clone(),
                        cube_shadow_maps.clone(),
                        spec.shader_type,
                        false,
                    );
                }
            }
        }
    }

    /// renders the shadows to the shadow map
    fn render_shadows(&mut self) {
        //
        // directional lights
        //
        let mut current_renderer_arch = None;
        for (_, shadow_map) in self.directional_lights.iter_mut() {
            shadow_map.bind_writing();
            for renderer_type in self.renderers.iter_mut() {
                let new_arch = renderer_type.required_shader().arch;
                if current_renderer_arch
                    .map(|arch| arch != new_arch)
                    .unwrap_or(true)
                {
                    current_renderer_arch = Some(new_arch);
                    self.shader_catalog
                        .use_shadow_shader(current_renderer_arch.unwrap(), false);
                    shadow_map.bind_light_matrix();
                }
                match renderer_type {
                    RendererType::Batch { renderer, .. } => {
                        renderer.render_shadows();
                    }
                    RendererType::Instance { renderer, .. } => {
                        renderer.render_shadows();
                    }
                }
            }
            shadow_map.unbind_writing();
        }
        //
        // point lights
        //
        current_renderer_arch = None;
        for shadow_cube_map in self
            .point_lights
            .values_mut()
            .filter_map(|info| info.shadow_map.as_mut())
        {
            shadow_cube_map.bind_writing();
            for renderer_type in self.renderers.iter_mut() {
                let new_arch = renderer_type.required_shader().arch;
                if current_renderer_arch
                    .map(|arch| arch != new_arch)
                    .unwrap_or(true)
                {
                    current_renderer_arch = Some(new_arch);
                    self.shader_catalog
                        .use_shadow_shader(current_renderer_arch.unwrap(), true);
                    shadow_cube_map.bind_light_uniforms();
                }
                match renderer_type {
                    RendererType::Batch { renderer, .. } => {
                        renderer.render_cube_shadows();
                    }
                    RendererType::Instance { renderer, .. } => {
                        renderer.render_cube_shadows();
                    }
                }
            }
            shadow_cube_map.unbind_writing();
        }
    }

    /// try to add the render data to an existing renderer
    fn try_add_data(&mut self, rd: &RenderData) -> bool {
        for r_type in self.renderers.iter_mut() {
            if let RendererType::Batch {
                spec,
                renderer,
                used,
                transp,
            } = r_type
            {
                if *spec == rd.spec {
                    *transp = *transp || rd.transparent;
                    *used = true;
                    return match rd.m_attr {
                        MeshAttribute::Textured(path) => {
                            renderer.draw_tex_mesh(
                                rd.trafo,
                                rd.tex_map.get_tex_id(path).unwrap(),
                                rd.mesh,
                                spec.shader_type,
                            );
                            true
                        }
                        MeshAttribute::Colored(color) => {
                            renderer.draw_color_mesh(
                                rd.trafo,
                                *color,
                                rd.mesh,
                                rd.spec.shader_type,
                            );
                            true
                        }
                    };
                }
            } else if let RendererType::Instance {
                spec,
                attribute,
                renderer,
                used,
                transp,
            } = r_type
            {
                if *spec == rd.spec && attribute == rd.m_attr {
                    match rd.m_attr {
                        MeshAttribute::Textured(path) => {
                            if rd.tex_map.get_tex_id(path).unwrap() == renderer.tex_id {
                                renderer.add_position(rd.trafo, rd.mesh);
                                *transp = *transp || rd.transparent;
                                *used = true;
                                return true;
                            }
                        }
                        MeshAttribute::Colored(color) => {
                            if *color == renderer.color {
                                renderer.add_position(rd.trafo, rd.mesh);
                                *transp = *transp || rd.transparent;
                                *used = true;
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// add a new renderer to the system and add the render data to it
    fn add_new_renderer(&mut self, rd: &RenderData) {
        match rd.spec.mesh_type {
            MeshType::Triangle | MeshType::Plane | MeshType::Cube => {
                let mut renderer = BatchRenderer::new();
                match rd.m_attr {
                    MeshAttribute::Colored(color) => {
                        renderer.draw_color_mesh(rd.trafo, *color, rd.mesh, rd.spec.shader_type);
                    }
                    MeshAttribute::Textured(path) => {
                        renderer.draw_tex_mesh(
                            rd.trafo,
                            rd.tex_map.get_tex_id(path).unwrap(),
                            rd.mesh,
                            rd.spec.shader_type,
                        );
                    }
                }
                self.renderers.push(RendererType::Batch {
                    spec: rd.spec,
                    renderer,
                    used: true,
                    transp: rd.transparent,
                });
                log::debug!(
                    "Added new BatchRenderer for mesh {:?} with LOD {:?} and shading style {:?}.",
                    rd.mesh.name,
                    rd.spec.lod,
                    rd.spec.shader_type
                );
            }
            _ => {
                let mut renderer = InstanceRenderer::new(rd.mesh, rd.spec.shader_type);
                match rd.m_attr {
                    MeshAttribute::Textured(path) => {
                        renderer.tex_id = rd.tex_map.get_tex_id(path).unwrap();
                    }
                    MeshAttribute::Colored(color) => {
                        renderer.color = *color;
                    }
                }
                renderer.add_position(rd.trafo, rd.mesh);
                self.renderers.push(RendererType::Instance {
                    spec: rd.spec,
                    attribute: rd.m_attr.clone(),
                    renderer,
                    used: true,
                    transp: rd.transparent,
                });
                log::debug!(
                    "Added new InstanceRenderer for mesh {:?} with LOD {:?} and shading style {:?}.",
                    rd.mesh.name,
                    rd.spec.lod,
                    rd.spec.shader_type
                );
            }
        }
        self.renderers.sort_unstable();
    }

    /// updates all the uniform buffers
    fn update_uniform_buffers(&self) {
        self.shader_catalog.matrix_buffer.upload_data(
            0,
            size_of::<Mat4>(),
            &self.perspective_camera.projection as *const Mat4 as *const GLvoid,
        );
        self.shader_catalog.matrix_buffer.upload_data(
            size_of::<Mat4>(),
            size_of::<Mat4>(),
            &self.perspective_camera.view as *const Mat4 as *const GLvoid,
        );
        let cam_pos_4 = to_vec4(&self.current_cam_config.0);
        self.shader_catalog.matrix_buffer.upload_data(
            size_of::<Mat4>() * 2,
            size_of::<Vec4>(),
            &cam_pos_4 as *const Vec4 as *const GLvoid,
        );

        self.shader_catalog.ortho_buffer.upload_data(
            0,
            size_of::<Mat4>(),
            &self.ortho_camera.projection as *const Mat4 as *const GLvoid,
        );
        self.shader_catalog.ortho_buffer.upload_data(
            size_of::<Mat4>(),
            size_of::<Mat4>(),
            &self.ortho_camera.view as *const Mat4 as *const GLvoid,
        );

        let dir_light_data = self
            .directional_lights
            .iter()
            .map(|(_, map)| DirLightData {
                light_pos: to_vec4(&map.light_pos),
                light_matrix: map.light_matrix,
                color: map.light.color.to_vec4(),
                intensity: map.light.intensity,
                padding_12bytes: Default::default(),
                direction: map.light.direction.normalize(),
                padding_4bytes: Default::default(),
            })
            .collect_vec();

        let p_light_data = self
            .point_lights
            .values()
            .map(|render_info| PointLightData {
                light_pos: to_vec4(&render_info.light_pos),
                color: render_info.light.color.to_vec4(),
                intensity: render_info.light.intensity,
                has_shadows: render_info.shadow_map.is_some() as GLint,
                padding_8bytes: Default::default(),
            })
            .collect_vec();

        let ambient_config = LightConfig {
            color: self.ambient_light.0.to_vec4(),
            intensity: self.ambient_light.1,
        };

        self.shader_catalog.light_buffer.upload_data(
            0,
            size_of::<LightConfig>(),
            &ambient_config as *const LightConfig as *const GLvoid,
        );

        if !dir_light_data.is_empty() {
            self.shader_catalog.light_buffer.upload_data(
                size_of::<LightConfig>() + padding::<LightConfig>(),
                dir_light_data.len() * size_of::<DirLightData>(),
                dir_light_data.as_ptr() as *const GLvoid,
            );
        }
        if !p_light_data.is_empty() {
            self.shader_catalog.light_buffer.upload_data(
                size_of::<LightConfig>()
                    + padding::<LightConfig>()
                    + size_of::<DirLightData>() * MAX_DIR_LIGHT_MAPS,
                p_light_data.len() * size_of::<PointLightData>(),
                p_light_data.as_ptr() as *const GLvoid,
            );
        }

        let num_dir_lights = self.directional_lights.len() as GLint;
        let num_point_lights = self.point_lights.len() as GLint;

        self.shader_catalog.light_buffer.upload_data(
            size_of::<LightConfig>()
                + padding::<LightConfig>()
                + size_of::<DirLightData>() * MAX_DIR_LIGHT_MAPS
                + size_of::<PointLightData>() * MAX_POINT_LIGHT_COUNT,
            size_of::<GLint>(),
            &num_dir_lights as *const GLint as *const GLvoid,
        );
        self.shader_catalog.light_buffer.upload_data(
            size_of::<LightConfig>()
                + padding::<LightConfig>()
                + size_of::<DirLightData>() * MAX_DIR_LIGHT_MAPS
                + size_of::<PointLightData>() * MAX_POINT_LIGHT_COUNT
                + size_of::<GLint>(),
            size_of::<GLint>(),
            &num_point_lights as *const GLint as *const GLvoid,
        );
    }

    /// resets all renderers to the initial state
    fn reset_renderers(&mut self) {
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                RendererType::Batch {
                    renderer, transp, ..
                } => {
                    renderer.reset();
                    *transp = false;
                }
                RendererType::Instance {
                    renderer, transp, ..
                } => {
                    renderer.reset();
                    *transp = false;
                }
            }
        }
    }

    /// drop renderers that are not used anymore
    fn cleanup_renderers(&mut self) {
        self.renderers.retain(|r_type| r_type.used());
        for renderer_type in self.renderers.iter_mut() {
            match renderer_type {
                RendererType::Batch { renderer, .. } => {
                    renderer.clean_batches();
                }
                RendererType::Instance { .. } => {}
            }
        }
    }

    /// resets the usage flags of all renderers to false
    fn reset_renderer_usage(&mut self) {
        for renderer in self.renderers.iter_mut() {
            match renderer {
                RendererType::Batch { used, transp, .. } => {
                    *used = false;
                    *transp = false;
                }
                RendererType::Instance { used, transp, .. } => {
                    *used = false;
                    *transp = false;
                }
            }
        }
    }

    /// clears the OpenGL viewport
    fn clear_gl_screen(&self) {
        let float_color = self.clear_color.to_vec4();
        unsafe {
            gl::ClearColor(float_color.x, float_color.y, float_color.z, float_color.w);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    /// event listening function for window resizes
    pub(crate) fn update_viewport_ratio(&mut self, viewport_ratio: f32) {
        self.perspective_camera.update_win_size(viewport_ratio);
        self.ortho_camera.update_win_size(viewport_ratio);
    }

    /// Gets the current camera position, look and up direction vector.
    pub fn current_cam_config(&self) -> (Vec3, Vec3, Vec3) {
        self.current_cam_config
    }

    /// Sets the anti-aliasing mode for rendering (default is ``None``), samples should be 2, 4, or 8.
    pub fn set_msaa(&mut self, msaa_samples: Option<GLsizei>) {
        let use_msaa = msaa_samples.is_some();
        let msaa = unsafe { gl::IsEnabled(gl::MULTISAMPLE) == gl::TRUE };
        if msaa == use_msaa {
            return;
        }
        log::debug!("Set anti-aliasing: {use_msaa:?}.");
        if use_msaa {
            unsafe { gl::Enable(gl::MULTISAMPLE) };
            self.screen_texture.msaa = true;
        } else {
            unsafe { gl::Disable(gl::MULTISAMPLE) };
            self.screen_texture.msaa = false;
        }
        self.samples = msaa_samples.unwrap_or(4);
    }

    /// Sets the resolution that is used for the screen texture in 3D rendering (width, height).
    pub fn set_3d_render_resolution(&mut self, resolution: (GLsizei, GLsizei)) {
        let msaa = unsafe { gl::IsEnabled(gl::MULTISAMPLE) == gl::TRUE };
        self.screen_texture = ScreenTexture::new(resolution.0, resolution.1, msaa, self.samples);
        log::debug!("Set 3D render resolution to {resolution:?}.");
    }

    /// Sets the current skybox that is used in the rendering process (default is ``None``).
    pub fn set_skybox(&mut self, skybox: Option<Skybox>) {
        self.skybox = skybox;
    }

    /// Access to the sprite grid config of the given layer.
    pub fn sprite_grid_mut(&mut self, layer: SpriteLayer) -> &mut SpriteGrid {
        &mut self.sprite_renderer.grids[layer as usize]
    }

    /// Changes the rendering backend's background clear color (default is WHITE).
    pub fn set_gl_clearcolor(&mut self, color: Color32) {
        self.clear_color = color;
        log::trace!("Set background clear color: {color:?}.");
    }

    /// Sets the FOV for 3D rendering in degrees (default is 45°).
    pub fn set_fov(&mut self, fov: f32) {
        self.perspective_camera.update_fov(fov);
        log::trace!("Set FOV: {fov:?}.");
    }

    /// Changes the render distance to `distance` units from the current camera position.
    pub fn set_render_distance(&mut self, distance: Option<f32>) {
        self.render_distance = distance;
        log::debug!("Set render distance: {distance:?}.");
    }

    /// Changes the shadow map resolution (default is normal).
    pub fn set_shadow_resolution(&mut self, resolution: ShadowResolution) {
        self.shadow_resolution = resolution;
        self.directional_lights.iter_mut().for_each(|(_, map)| {
            *map = ShadowMap::new(self.shadow_resolution.map_res(), map.light_pos, &map.light)
        });
        self.point_lights
            .values_mut()
            .filter_map(|info| {
                info.shadow_map
                    .as_mut()
                    .map(|map| (info.light_pos, &info.light, map))
            })
            .for_each(|(pos, _, map)| {
                *map = CubeShadowMap::new(self.shadow_resolution.map_res(), pos)
            });
        log::debug!("Set shadow map resolution: {resolution:?}.");
    }

    /// Changes the ambient light (default is white and 0.2).
    pub fn set_ambient_light(&mut self, color: Color32, intensity: f32) {
        self.ambient_light = (color, intensity);
        log::debug!("Set ambient light to {color:?} with intensity {intensity:?}.");
    }

    pub(crate) fn on_cam_position_change(&mut self, event: &CamPositionChange) {
        let new_focus = event.new_pos + event.new_look;
        self.perspective_camera
            .update_cam(&event.new_pos, &new_focus, &event.new_up);
        self.current_cam_config = (event.new_pos, event.new_look, event.new_up);
    }
}

/// specifies what renderer to use for rendering an entity
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
struct RenderSpec {
    mesh_type: MeshType,
    shader_type: ShaderType,
    lod: LOD,
}

/// all variants of renderer architecture
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum RendererArch {
    Batch,
    Instance,
}

/// identifies a shader (combines renderer architecture and shader type)
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct ShaderSpec {
    pub(crate) arch: RendererArch,
    pub(crate) shader_type: ShaderType,
}

/// stores the renderer type with rendered entity type + renderer
enum RendererType {
    Batch {
        spec: RenderSpec,
        renderer: BatchRenderer,
        used: bool,
        transp: bool,
    },
    Instance {
        spec: RenderSpec,
        attribute: MeshAttribute,
        renderer: InstanceRenderer,
        used: bool,
        transp: bool,
    },
}

impl RendererType {
    /// returns the value of the renderers use flag
    fn used(&self) -> bool {
        match self {
            RendererType::Batch { used, .. } => *used,
            RendererType::Instance { used, .. } => *used,
        }
    }

    /// returns the value of the renderers transparency flag
    fn transparent(&self) -> bool {
        match self {
            RendererType::Batch { transp, .. } => *transp,
            RendererType::Instance { transp, .. } => *transp,
        }
    }

    /// returns the shader requirement for this shader
    fn required_shader(&self) -> ShaderSpec {
        match self {
            RendererType::Batch { spec, .. } => ShaderSpec {
                arch: RendererArch::Batch,
                shader_type: spec.shader_type,
            },
            RendererType::Instance { spec, .. } => ShaderSpec {
                arch: RendererArch::Instance,
                shader_type: spec.shader_type,
            },
        }
    }
}

impl Eq for RendererType {}

impl PartialEq<Self> for RendererType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RendererType::Batch { spec: spec1, .. } => match other {
                RendererType::Batch { spec: spec2, .. } => spec1.shader_type == spec2.shader_type,
                RendererType::Instance { .. } => false,
            },
            RendererType::Instance { spec: spec1, .. } => match other {
                RendererType::Batch { .. } => false,
                RendererType::Instance { spec: spec2, .. } => {
                    spec1.shader_type == spec2.shader_type
                }
            },
        }
    }
}

impl PartialOrd<Self> for RendererType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RendererType {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            RendererType::Batch { spec: spec1, .. } => match other {
                RendererType::Batch { spec: spec2, .. } => {
                    spec1.shader_type.cmp(&spec2.shader_type)
                }
                RendererType::Instance { .. } => Ordering::Less,
            },
            RendererType::Instance { spec: spec1, .. } => match other {
                RendererType::Batch { .. } => Ordering::Greater,
                RendererType::Instance { spec: spec2, .. } => {
                    spec1.shader_type.cmp(&spec2.shader_type)
                }
            },
        }
    }
}

/// data bundle for rendering
struct RenderData<'a> {
    spec: RenderSpec,
    trafo: &'a Mat4,
    m_attr: &'a MeshAttribute,
    mesh: &'a Mesh,
    tex_map: &'a TextureMap,
    transparent: bool,
}

/// All possible settings for shadow map resolution.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShadowResolution {
    Ultra,
    High,
    Normal,
    Low,
}

impl ShadowResolution {
    /// Yields the actual corresponding map resolution to the setting.
    pub fn map_res(&self) -> (GLsizei, GLsizei) {
        match self {
            ShadowResolution::Ultra => (4096, 4096),
            ShadowResolution::High => (2048, 2048),
            ShadowResolution::Normal => (1024, 1024),
            ShadowResolution::Low => (512, 512),
        }
    }
}

/// general temporary storage used in the update process of the renderer
#[derive(Default)]
struct TempRenderStorage {
    dir_lights: Vec<(Position, DirectionalLight, EntityID)>,
    p_lights: Vec<(Position, PointLight, EntityID)>,
}

impl TempRenderStorage {
    /// clears the light source storage vectors
    fn clear_light_storage(&mut self) {
        self.dir_lights.clear();
        self.p_lights.clear();
    }
}
