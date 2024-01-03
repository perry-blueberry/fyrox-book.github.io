use fyrox::{
    core::{
        algebra::{UnitQuaternion, UnitVector3, Vector3},
        pool::Handle,
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*,
        TypeUuidProvider,
    },
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    impl_component_provider,
    keyboard::{KeyCode, PhysicalKey},
    scene::{node::Node, rigidbody::RigidBody},
    script::{ScriptContext, ScriptTrait},
};

#[derive(Visit, Reflect, Default, Debug, Clone)]
pub struct Player {
    // ANCHOR: input_fields
    #[visit(optional)]
    #[reflect(hidden)]
    move_forward: bool,

    #[visit(optional)]
    #[reflect(hidden)]
    move_backward: bool,

    #[visit(optional)]
    #[reflect(hidden)]
    move_left: bool,

    #[visit(optional)]
    #[reflect(hidden)]
    move_right: bool,

    #[visit(optional)]
    #[reflect(hidden)]
    yaw: f32,

    #[visit(optional)]
    #[reflect(hidden)]
    pitch: f32,
    // ANCHOR_END: input_fields

    // ANCHOR: camera_field
    #[visit(optional)]
    camera: Handle<Node>,
    // ANCHOR_END: camera_field
}

impl_component_provider!(Player);

impl TypeUuidProvider for Player {
    fn type_uuid() -> Uuid {
        uuid!("49cfe29e-c7c7-4317-8178-996251a0c2f9")
    }
}

impl ScriptTrait for Player {
    // ANCHOR: on_os_event
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        match event {
            // Raw mouse input is responsible for camera rotation.
            Event::DeviceEvent {
                event:
                    DeviceEvent::MouseMotion {
                        delta: (dx, dy), ..
                    },
                ..
            } => {
                // Pitch is responsible for vertical camera rotation. It has -89.9..89.0 degree limits,
                // to prevent infinite rotation.
                let mouse_speed = 0.35;
                self.pitch = (self.pitch + *dy as f32 * mouse_speed).clamp(-89.9, 89.9);
                self.yaw -= *dx as f32 * mouse_speed;
            }
            // Keyboard input is responsible for player's movement.
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let is_pressed = event.state == ElementState::Pressed;
                    match code {
                        KeyCode::KeyW => {
                            self.move_forward = is_pressed;
                        }
                        KeyCode::KeyS => {
                            self.move_backward = is_pressed;
                        }
                        KeyCode::KeyA => {
                            self.move_left = is_pressed;
                        }
                        KeyCode::KeyD => {
                            self.move_right = is_pressed;
                        }
                        _ => (),
                    }
                }
            }
            _ => {}
        }
    }
    // ANCHOR_END: on_os_event

    // ANCHOR: on_update
    fn on_update(&mut self, context: &mut ScriptContext) {
        // ANCHOR: camera_rotation
        let mut look_vector = Vector3::default();
        let mut side_vector = Vector3::default();
        if let Some(camera) = context.scene.graph.try_get_mut(self.camera) {
            look_vector = camera.look_vector();
            side_vector = camera.side_vector();

            let yaw = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.yaw.to_radians());
            let transform = camera.local_transform_mut();
            transform.set_rotation(
                UnitQuaternion::from_axis_angle(
                    &UnitVector3::new_normalize(yaw * Vector3::x()),
                    self.pitch.to_radians(),
                ) * yaw,
            );
        }
        // ANCHOR_END: camera_rotation

        // Borrow the node to which this script is assigned to. We also check if the node is RigidBody.
        if let Some(rigid_body) = context
            .scene
            .graph
            .try_get_mut_of_type::<RigidBody>(context.handle)
        {
            // Form a new velocity vector that corresponds to the pressed buttons.
            let mut velocity = Vector3::new(0.0, 0.0, 0.0);
            if self.move_forward {
                velocity += look_vector;
            }
            if self.move_backward {
                velocity -= look_vector;
            }
            if self.move_left {
                velocity += side_vector;
            }
            if self.move_right {
                velocity -= side_vector;
            }

            let y_vel = rigid_body.lin_vel().y;
            if let Some(normalized_velocity) = velocity.try_normalize(f32::EPSILON) {
                let movement_speed = 240.0 * context.dt;
                rigid_body.set_lin_vel(Vector3::new(
                    normalized_velocity.x * movement_speed,
                    y_vel,
                    normalized_velocity.z * movement_speed,
                ));
            } else {
                // Hold player in-place in XZ plane when no button is pressed.
                rigid_body.set_lin_vel(Vector3::new(0.0, y_vel, 0.0));
            }
        }
    }
    // ANCHOR_END: on_update

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}