#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    y: isize,
    x: isize,
}

struct Moving {
    direction: Direction,
    step_timer: Timer,
}

struct Health {
    lives: usize,
    max_health: usize,
    health: usize,
}

struct BaseMaterial(Handle<ColorMaterial>);
struct ImmortalMaterial(Handle<ColorMaterial>);

struct Bomb {
    parent: Entity,
    range: usize,
}

struct Fuse {}

struct BombSatchel {
    bombs_available: usize,
    bomb_range: usize,
}

struct Immortal {
    timer: Timer,
}

#[derive(Bundle)]
struct ImmortalBundle {
    immortal: Immortal,
    animation_timer: Timer,
}

impl Default for ImmortalBundle {
    fn default() -> Self {
        ImmortalBundle {
            immortal: Immortal {
                timer: Timer::from_seconds(2.0, false),
            },
            animation_timer: Timer::from_seconds(0.66, true),
        }
    }
}

struct WallHack;

struct BombPush;

struct MeleeAttacker;

struct TeamAlignment(usize);

struct Perishable {
    timer: Timer,
}

struct Fire;

struct Solid;

struct Wall;

struct Destructible;

// Events

// position + range
struct PlayerActionEvent(Entity, PlayerAction);

#[derive(Clone, Copy)]
struct ExplosionEvent(Position, usize);

struct DamageEvent(Entity);

struct BurnEvent(Position);

// Camera that adjusts to window size + maintains aspect ratio

struct SimpleOrthoProjection {
    far: f32,
    aspect: f32,
    flag: bool,
    multiplier: f32,
    perfect_aspect_ratio: f32,
    map_pixel_width: f32,
    map_pixel_height: f32,
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
            0.0,
            self.far,
        )
    }

    // what to do on window resize
    fn update(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
        self.flag = self.aspect > self.perfect_aspect_ratio;
        self.multiplier = if self.flag {
            self.map_pixel_width
        } else {
            self.map_pixel_height
        };
    }

    fn depth_calculation(&self) -> DepthCalculation {
        DepthCalculation::ZDifference
    }
}

impl SimpleOrthoProjection {
    fn new(width: usize, height: usize) -> Self {
        let map_pixel_height = (TILE_WIDTH * width) as f32;
        let map_pixel_width = (TILE_HEIGHT * height) as f32;

        println!("h: {}, w: {}", map_pixel_height, map_pixel_width);

        let perfect_aspect_ratio = map_pixel_height / map_pixel_width;

        SimpleOrthoProjection {
            far: 1000.0,
            aspect: 1.0,
            flag: true,
            multiplier: map_pixel_height,
            perfect_aspect_ratio,
            map_pixel_width,
            map_pixel_height,
        }
    }
}
