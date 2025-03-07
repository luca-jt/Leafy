use crate::ecs::component::utils::TimeDuration;
use crate::ecs::component::{EntityFlags, Position, SoundController};
use crate::engine::{Engine, FallingLeafApp};
use crate::glm;
use crate::systems::event_system::events::user_space::*;
use crate::systems::event_system::events::*;
use crate::utils::constants::bits::user_level::DOPPLER_EFFECT;
use crate::utils::constants::Y_AXIS;
use crate::utils::tools::map_range;
use std::f32::consts::{FRAC_PI_2, PI};

/// starts moving the camera in the direction the key was pressed for
pub(crate) fn move_cam<T: FallingLeafApp>(event: &KeyPress, engine: &Engine<T>) {
    if event.is_repeat {
        return;
    }
    let keys = engine.animation_system().flying_cam_keys;
    if let Some((cam_move_direction, _)) = engine.animation_system_mut().flying_cam_dir.as_mut() {
        if event.key == keys.down {
            cam_move_direction.y -= 1.0;
        }
        if event.key == keys.up {
            cam_move_direction.y += 1.0;
        }
        if event.key == keys.forward {
            cam_move_direction.z += 1.0;
        }
        if event.key == keys.left {
            cam_move_direction.x -= 1.0;
        }
        if event.key == keys.backward {
            cam_move_direction.z -= 1.0;
        }
        if event.key == keys.right {
            cam_move_direction.x += 1.0;
        }
    }
}

/// stops the cam form moving in the direction the key was released for
pub(crate) fn stop_cam<T: FallingLeafApp>(event: &KeyRelease, engine: &Engine<T>) {
    if event.is_repeat {
        return;
    }
    let keys = engine.animation_system().flying_cam_keys;
    if let Some((cam_move_direction, _)) = engine.animation_system_mut().flying_cam_dir.as_mut() {
        if event.key == keys.down {
            cam_move_direction.y += 1.0;
        }
        if event.key == keys.up {
            cam_move_direction.y -= 1.0;
        }
        if event.key == keys.forward {
            cam_move_direction.z -= 1.0;
        }
        if event.key == keys.left {
            cam_move_direction.x += 1.0;
        }
        if event.key == keys.backward {
            cam_move_direction.z += 1.0;
        }
        if event.key == keys.right {
            cam_move_direction.x -= 1.0;
        }
    }
}

/// enables 3D camera control with the mouse if the required setting is enabled
pub(crate) fn mouse_move_cam<T: FallingLeafApp>(event: &RawMouseMotion, engine: &Engine<T>) {
    if let Some(sens) = engine.video_system().mouse_cam_sens {
        let cam_config = engine.rendering_system().current_cam_config();
        debug_assert!(
            cam_config.1 != Y_AXIS && cam_config.1 != -Y_AXIS && cam_config.1.norm() > 0.0,
            "viewing angle must be in interval (-pi, pi] and look vector cannot have length 0"
        );
        let look_dir = cam_config.1.normalize(); // new z
        let right_dir = look_dir.cross(&Y_AXIS).normalize(); // new x
        let up_dir = right_dir.cross(&look_dir).normalize(); // new y
        let look_trafo = glm::Mat3::from_columns(&[right_dir, up_dir, look_dir]);

        let forward_dir = glm::vec3(look_dir.x, 0.0, look_dir.z);
        let forward_dir_norm = forward_dir.norm();
        let current_vert_angle = forward_dir_norm.acos();
        let add_angle = sens / 1000.0;

        let hori_factor = forward_dir_norm; // accounts for different circle radii when the vertical angle changes
        let add_hori_angle = add_angle * event.delta_x as f32 * hori_factor;
        let look_hori = look_trafo * glm::vec3(add_hori_angle.sin(), 0.0, add_hori_angle.cos());

        let angle_block = PI / 16.0;
        let add_vert_angle = (add_angle * -event.delta_y as f32).clamp(
            -FRAC_PI_2 + angle_block + current_vert_angle,
            FRAC_PI_2 - angle_block - current_vert_angle,
        );
        let look_vert = look_trafo * glm::vec3(0.0, add_vert_angle.sin(), add_vert_angle.cos());

        engine.trigger_event(CamPositionChange {
            new_pos: cam_config.0,
            new_look: (look_hori + look_vert).normalize(),
        });
    }
}

/// updates the current engine mode and all systems that are influenced by that
pub(crate) fn on_mode_change<T: FallingLeafApp>(event: &EngineModeChange, engine: &Engine<T>) {
    engine.mode.set(event.new_mode);
    engine.audio_system_mut().on_mode_change(event);
}

/// updates the camera position based on the current movement key induced camera movement
pub(crate) fn update_cam<T: FallingLeafApp>(engine: &Engine<T>, dt: TimeDuration) {
    if engine.animation_system().flying_cam_dir.is_none() {
        return;
    }
    let cam_move_config = engine.animation_system().flying_cam_dir.unwrap();
    if cam_move_config.0 != glm::Vec3::zeros() {
        let cam_config = engine.rendering_system().current_cam_config();
        let move_vector = cam_move_config.0.normalize();
        let changed = move_vector * dt.0 * cam_move_config.1;

        let mut look_z = cam_config.1;
        look_z.y = 0.0;
        look_z.normalize_mut();
        let look_x = look_z.cross(&Y_AXIS).normalize();
        let look_space_matrix = glm::Mat3::from_columns(&[look_x, Y_AXIS, look_z]);

        engine.trigger_event(CamPositionChange {
            new_pos: cam_config.0 + look_space_matrix * changed,
            new_look: cam_config.1,
        });
    }
}

/// updates the doppler effect data for the audio system
pub(crate) fn update_doppler_data<T: FallingLeafApp>(engine: &Engine<T>, dt: TimeDuration) {
    let mut animation_system = engine.animation_system_mut();
    for (pos, sound, flags_opt) in unsafe {
        engine
            .entity_manager()
            .query3::<&Position, &mut SoundController, Option<&EntityFlags>>((None, None))
    } {
        let doppler_effect = flags_opt.is_some_and(|f| f.get_bit(DOPPLER_EFFECT));

        let entity_vel = (pos.data() - sound.last_pos) / dt.0;
        sound.last_pos = *pos.data();
        let cam_vel = (animation_system.curr_cam_pos - animation_system.prev_cam_pos) / dt.0;
        animation_system.prev_cam_pos = animation_system.curr_cam_pos;

        if doppler_effect {
            let rel_vel = entity_vel - cam_vel;
            let to_cam = (animation_system.curr_cam_pos - pos.data()).normalize();
            let doppler_coeff = to_cam.dot(&rel_vel).clamp(-256.0, 256.0);
            let pitch = if doppler_coeff < 0.0 {
                map_range((-256.0, 0.0), (0.5, 1.0), doppler_coeff) as f64
            } else if doppler_coeff > 0.0 {
                map_range((0.0, 256.0), (1.0, 2.0), doppler_coeff) as f64
            } else {
                1.0
            };
            engine
                .audio_system()
                .set_doppler_pitch(sound, sound.doppler_pitch, pitch);
            sound.doppler_pitch = pitch;
        } else {
            let pitch = 1.0;
            engine
                .audio_system()
                .set_doppler_pitch(sound, sound.doppler_pitch, pitch);
            sound.doppler_pitch = pitch;
        }
    }
}

/// general event handling function for the window resize
pub(crate) fn on_window_resize<T: FallingLeafApp>(event: &WindowResize, engine: &Engine<T>) {
    let mut video_system = engine.video_system_mut();
    let mut rendering_system = engine.rendering_system_mut();
    video_system.on_window_resize(event);
    let viewport_ratio = video_system.current_viewport_ratio();
    rendering_system.update_viewport_ratio(viewport_ratio);
}

/// general event handling function for the animation speed change
pub(crate) fn on_animation_speed_change<T: FallingLeafApp>(
    event: &AnimationSpeedChange,
    engine: &Engine<T>,
) {
    engine
        .animation_system_mut()
        .on_animation_speed_change(event);
    engine.audio_system_mut().on_animation_speed_change(event);
}

/// general event handling function for the camera position change
pub(crate) fn on_cam_position_change<T: FallingLeafApp>(
    event: &CamPositionChange,
    engine: &Engine<T>,
) {
    engine.animation_system_mut().on_cam_position_change(event);
    engine.audio_system_mut().on_cam_position_change(event);
    engine.rendering_system_mut().on_cam_position_change(event);
}
