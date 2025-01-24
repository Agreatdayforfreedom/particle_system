use crate::uniform::Uniform;
use cgmath::{Matrix4, Rad, Vector2, Vector3};
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::window::InputEvent;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
const SAFE_MIN_RADIUS: f32 = 1.0;
pub struct Camera2D {
    pub position: Vector3<f32>,
    pub scale: Vector2<f32>,
    pub uniform: Uniform<Camera2DUniform>,
}

impl Camera2D {
    pub fn new(uniform: Uniform<Camera2DUniform>) -> Self {
        Self {
            uniform,
            scale: (WIDTH, HEIGHT).into(),
            position: (0.0, 0.0, 0.0).into(),
        }
    }
    pub fn update(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.uniform.data.update(position);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera2DUniform {
    pub proj: [[f32; 4]; 4],
}
impl Camera2DUniform {
    fn update(&mut self, position: Vector3<f32>) {
        let view = Matrix4::from_translation(-position);
        let ortho = OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -WIDTH / 2.0,
                WIDTH / 2.0,
                -HEIGHT / 2.0,
                HEIGHT / 2.0,
                -50.0,
                50.0,
            );
        self.proj = (ortho * view).into();
    }
}

impl Default for Camera2DUniform {
    fn default() -> Self {
        let position = Vector3::new(0.0, 0.0, 1.0);

        let view = Matrix4::from_translation(-position);
        let ortho = cgmath::ortho(0.0, 800.0, 600.0, 0.0, -50.0, 50.0);
        Self {
            proj: (ortho * view).into(),
        }
    }
}

pub struct Camera3D {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    radius: f32,
    theta: Rad<f32>,
    phi: Rad<f32>,
    pub uniform: Uniform<Camera3DUniform>,
}

impl Camera3D {
    pub fn new(uniform: Uniform<Camera3DUniform>) -> Self {
        Self {
            eye: (0.0, 0.0, 3.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: WIDTH / HEIGHT as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            theta: cgmath::Deg(-90.0).into(),
            phi: cgmath::Deg(-20.0).into(),
            radius: 90.0,
            uniform,
        }
    }
    pub fn build_view_projection_matrix(&mut self) {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        self.uniform.data.proj = (OPENGL_TO_WGPU_MATRIX * proj).into();
        self.uniform.data.view = view.into();
        self.uniform.data.position = self.eye.into();
    }

    pub fn update(&mut self, position: Vector3<f32>) {}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera3DUniform {
    pub proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub position: [f32; 3],
    _padding: u32,
}

impl Default for Camera3DUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            position: [0.0, 0.0, 0.0],
            _padding: 0,
        }
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,

    //mouse
    is_leftclick_pressed: bool,
    is_rightclick_pressed: bool,
    delta_x: f32,
    delta_y: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_leftclick_pressed: false,
            is_rightclick_pressed: false,
            delta_x: 0.0,
            delta_y: 0.0,
            sensitivity: 0.1,
        }
    }

    pub fn process_events(&mut self, event: InputEvent) -> bool {
        use winit::event::DeviceEvent;
        match event {
            InputEvent::Decive(device_event) => match device_event {
                DeviceEvent::MouseMotion { delta } => {
                    if self.is_leftclick_pressed || self.is_rightclick_pressed {
                        self.delta_x = delta.0 as f32;
                        self.delta_y = delta.1 as f32;
                    }
                    // returns false here because it does validates nothing
                    false
                }
                _ => false,
            },
            InputEvent::Window(window_event) => match window_event {
                WindowEvent::MouseInput { state, button, .. } => {
                    match button {
                        MouseButton::Left => self.is_leftclick_pressed = state.is_pressed(),
                        MouseButton::Right => self.is_rightclick_pressed = state.is_pressed(),
                        _ => {}
                    }
                    return true;
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state,
                            physical_key: PhysicalKey::Code(keycode),
                            ..
                        },
                    ..
                } => {
                    let is_pressed = *state == ElementState::Pressed;
                    match keycode {
                        KeyCode::KeyW | KeyCode::ArrowUp => {
                            self.is_forward_pressed = is_pressed;
                            true
                        }
                        KeyCode::KeyS | KeyCode::ArrowDown => {
                            self.is_backward_pressed = is_pressed;
                            true
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera3D) {
        use cgmath::InnerSpace;
        // calculate azimuth and polar angles
        if self.is_leftclick_pressed {
            camera.theta += Rad(self.delta_x * self.sensitivity * self.sensitivity);

            camera.phi += Rad(self.delta_y * self.sensitivity * self.sensitivity);
            if camera.phi < -Rad(SAFE_FRAC_PI_2) {
                camera.phi = -Rad(SAFE_FRAC_PI_2);
            } else if camera.phi > Rad(SAFE_FRAC_PI_2) {
                camera.phi = Rad(SAFE_FRAC_PI_2);
            }
        }

        // move the orbit target
        if self.is_rightclick_pressed {
            let forward = (camera.target - camera.eye).normalize();
            let right = forward.cross(camera.up).normalize();
            camera.target += right * (-self.delta_x * self.sensitivity)
                + camera.up * (self.delta_y * self.sensitivity);
        }

        // Set the deltas to 0.0, because if the mouse is not moving but the right or left click buttons are pressed, the camera will be updated anyway.
        self.delta_x = 0.0;
        self.delta_y = 0.0;

        // zoom
        if self.is_forward_pressed {
            let mut radius = camera.radius - 10.0 * self.sensitivity;
            if radius < SAFE_MIN_RADIUS {
                radius = SAFE_MIN_RADIUS;
            }
            camera.radius = radius;
        }
        if self.is_backward_pressed {
            let radius = camera.radius + 10.0 * self.sensitivity;
            camera.radius = radius;
        }

        camera.eye.x =
            camera.target.x + camera.radius * (camera.phi.0.cos() * camera.theta.0.cos());
        camera.eye.y = camera.target.y + camera.radius * camera.phi.0.sin();
        camera.eye.z =
            camera.target.z + camera.radius * (camera.phi.0.cos() * camera.theta.0.sin());
    }
}
