struct HumanControlledEntity(Entity);

#[derive(Default)]
struct Textures {
    empty: Handle<ColorMaterial>,
    penguin: Handle<ColorMaterial>,
    immortal_penguin: Handle<ColorMaterial>,
    crook: Handle<ColorMaterial>,
    immortal_crook: Handle<ColorMaterial>,
    hatter: Handle<ColorMaterial>,
    immortal_hatter: Handle<ColorMaterial>,
    bat: Handle<ColorMaterial>,
    immortal_bat: Handle<ColorMaterial>,
    bomb: Handle<ColorMaterial>,
    fire: Handle<ColorMaterial>,
    wall: Handle<ColorMaterial>,
    destructible_wall: Handle<ColorMaterial>,
    burning_wall: Handle<ColorMaterial>,
    burning_item: Handle<ColorMaterial>,
    bombs_up: Handle<ColorMaterial>,
    range_up: Handle<ColorMaterial>,
    lives_up: Handle<ColorMaterial>,
    wall_hack: Handle<ColorMaterial>,
    bomb_push: Handle<ColorMaterial>,
    immortal: Handle<ColorMaterial>,
}

#[derive(Default)]
struct Fonts {
    font1: Handle<Font>,
}
