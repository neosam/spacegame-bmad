use bevy::prelude::*;

/// Current frame input state. Systems read this to determine player intent.
/// Only `thrust` and `rotate` are populated in Story 0.1.
#[derive(Resource, Default, Debug)]
pub struct ActionState {
    /// 0.0–1.0 (trigger/key maps to 1.0)
    pub thrust: f32,
    /// -1.0–1.0 (stick analog, keys map to -1.0/1.0)
    pub rotate: f32,
    pub fire: bool,
    pub switch_weapon: bool,
    pub wingman_command: bool,
    pub interact: bool,
    pub toggle_map: bool,
    pub pause: bool,
}

/// Reads keyboard and gamepad input, writes to `ActionState` resource.
pub fn read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut action_state: ResMut<ActionState>,
) {
    // Reset each frame
    *action_state = ActionState::default();

    // Keyboard: thrust
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        action_state.thrust = 1.0;
    }

    // Keyboard: rotation
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        action_state.rotate += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        action_state.rotate -= 1.0;
    }

    // Keyboard: fire
    if keyboard.pressed(KeyCode::Space) {
        action_state.fire = true;
    }

    // Clamp rotation
    action_state.rotate = action_state.rotate.clamp(-1.0, 1.0);

    // Gamepad: override with analog values if available
    for gamepad in gamepads.iter() {
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let right_trigger = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);

        // Thrust: left stick up or right trigger
        let gamepad_thrust = left_stick_y.max(0.0).max(right_trigger);
        if gamepad_thrust > action_state.thrust {
            action_state.thrust = gamepad_thrust;
        }

        // Rotation: left stick X (negative = turn left = positive rotation)
        if left_stick_x.abs() > action_state.rotate.abs() {
            action_state.rotate = -left_stick_x;
        }

        // Fire: South button or right trigger threshold
        if gamepad.pressed(GamepadButton::South) || right_trigger > 0.5 {
            action_state.fire = true;
        }
    }

    // Final clamps
    action_state.thrust = action_state.thrust.clamp(0.0, 1.0);
    action_state.rotate = action_state.rotate.clamp(-1.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_state_defaults_to_zero() {
        let state = ActionState::default();
        assert_eq!(state.thrust, 0.0);
        assert_eq!(state.rotate, 0.0);
        assert!(!state.fire);
        assert!(!state.switch_weapon);
    }

    #[test]
    fn read_input_no_keys_produces_zero_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert_eq!(state.thrust, 0.0);
        assert_eq!(state.rotate, 0.0);
    }

    #[test]
    fn read_input_w_key_sets_thrust() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        // Press W key
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert_eq!(state.thrust, 1.0);
    }

    #[test]
    fn read_input_a_key_sets_positive_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert_eq!(state.rotate, 1.0);
    }

    #[test]
    fn read_input_space_sets_fire() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert!(state.fire, "Space key should set fire to true");
    }

    #[test]
    fn read_input_no_fire_key_leaves_fire_false() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        // Only press W (thrust), not Space
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert!(!state.fire, "Fire should be false when Space is not pressed");
    }

    #[test]
    fn read_input_d_key_sets_negative_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ActionState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, read_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        app.update();

        let state = app.world().resource::<ActionState>();
        assert_eq!(state.rotate, -1.0);
    }
}
