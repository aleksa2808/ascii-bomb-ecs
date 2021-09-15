use crate::types::RGBColor;

pub const PIXEL_SCALE: usize = 8;

pub const MENU_HEIGHT: usize = 100 * PIXEL_SCALE;
pub const MENU_WIDTH: usize = 100 * PIXEL_SCALE;

pub const HUD_HEIGHT: usize = 14 * PIXEL_SCALE;

pub const TILE_HEIGHT: usize = 8 * PIXEL_SCALE;
pub const TILE_WIDTH: usize = 6 * PIXEL_SCALE;

pub const SPLASH_SCREEN_TEXT_LEFT: &str = "Error 404:";
pub const SPLASH_SCREEN_TEXT_RIGHT: &str = "Name Not Found";

pub const DEMO_MODE_START_TIMER_DURATION_SECS: f32 = 15.0;

pub const COLORS: [RGBColor; 16] = [
    RGBColor(12, 12, 12),
    RGBColor(0, 55, 218),
    RGBColor(19, 161, 14),
    RGBColor(58, 150, 221),
    RGBColor(197, 15, 31),
    RGBColor(136, 23, 152),
    RGBColor(193, 156, 0),
    RGBColor(204, 204, 204),
    RGBColor(118, 118, 118),
    RGBColor(59, 120, 255),
    RGBColor(22, 198, 12),
    RGBColor(97, 214, 214),
    RGBColor(231, 72, 86),
    RGBColor(180, 0, 158),
    RGBColor(249, 241, 165),
    RGBColor(242, 242, 242),
];
