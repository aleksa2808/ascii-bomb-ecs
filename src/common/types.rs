use bevy::prelude::Color;

#[derive(Clone, Copy)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl From<RGBColor> for Color {
    fn from(color: RGBColor) -> Self {
        Self::rgb_u8(color.0, color.1, color.2)
    }
}
