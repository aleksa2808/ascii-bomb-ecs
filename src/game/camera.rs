use bevy::{
    ecs as bevy_ecs,
    prelude::*,
    render::camera::{CameraProjection, DepthCalculation},
};

// Camera that adjusts to window size + maintains aspect ratio

#[derive(Component)]
pub struct SimpleOrthoProjection {
    pub near: f32,
    far: f32,
    aspect: f32,
    flag: bool,
    multiplier: f32,
    perfect_aspect_ratio: f32,
    map_pixel_height: f32,
    map_pixel_width: f32,
}

impl CameraProjection for SimpleOrthoProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        let (right, bottom) = if self.flag {
            (self.aspect, 1.0)
        } else {
            (1.0, 1.0 / self.aspect)
        };

        Mat4::orthographic_rh(
            0.0,
            right * self.multiplier,
            -bottom * self.multiplier,
            0.0,
            self.near,
            self.far,
        )
    }

    // what to do on window resize
    fn update(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
        self.flag = self.aspect > self.perfect_aspect_ratio;
        self.multiplier = if self.flag {
            self.map_pixel_height
        } else {
            self.map_pixel_width
        };
    }

    fn depth_calculation(&self) -> DepthCalculation {
        DepthCalculation::ZDifference
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl SimpleOrthoProjection {
    pub fn new(map_pixel_height: f32, map_pixel_width: f32) -> Self {
        let perfect_aspect_ratio = map_pixel_width / map_pixel_height;
        SimpleOrthoProjection {
            near: 0.0,
            far: 1000.0,
            aspect: 1.0,
            flag: true,
            multiplier: map_pixel_width,
            perfect_aspect_ratio,
            map_pixel_width,
            map_pixel_height,
        }
    }
}
