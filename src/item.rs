use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Power {
    WallHack,
    BombPush,
    Immortal,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum Upgrade {
    BombsUp,
    RangeUp,
    LivesUp,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Upgrade(Upgrade),
    Power(Power),
}

impl Item {
    pub fn generate(reduced_loot: bool) -> Self {
        let r = rand::thread_rng().gen::<usize>() % 100;

        /* "Loot tables" */
        if !reduced_loot {
            match r {
                _ if r < 50 => Item::Upgrade(Upgrade::BombsUp),
                50..=79 => Item::Upgrade(Upgrade::RangeUp),
                80..=89 => Item::Power(Power::BombPush),
                90..=93 => Item::Upgrade(Upgrade::LivesUp),
                94..=97 => Item::Power(Power::WallHack),
                _ if r >= 98 => Item::Power(Power::Immortal),
                _ => unreachable!(),
            }
        } else {
            match r {
                _ if r < 50 => Item::Upgrade(Upgrade::BombsUp),
                50..=89 => Item::Upgrade(Upgrade::RangeUp),
                _ if r >= 90 => Item::Power(Power::BombPush),
                _ => unreachable!(),
            }
        }
    }
}
