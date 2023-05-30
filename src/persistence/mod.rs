use crate::{
    character::{
        player::{Player, Skills, Stats},
        Gender, OutfitColors,
    },
    map::MAP,
};
use std::sync::atomic::{AtomicU32, Ordering};

fn get_player_id(_name: &str) -> Option<u32> {
    static NEXT_ID: AtomicU32 = AtomicU32::new(256);
    Some(NEXT_ID.fetch_add(1, Ordering::SeqCst))
}

pub fn load_player_by_name(name: &str) -> Option<Player> {
    get_player_id(name).map(|id| {
        Player {
            id,
            name: name.to_owned(),
            position: MAP.get().unwrap().metadata.respawn_location,
            skills: Skills {
                sword: 10,
                club: 10,
                axe: 10,
                distance: 10, //on v4 this is 'throwing'
                shield: 10,
                fist: 10,
                fishing: 10,

                //only on v4
                gauche: 10,
                missile: 10,
            },
            stats: Stats {
                health_points: 150,
                capacity: 400,
                intelligence: 10,
                strength: 10,
                dexterity: 10,
                experience_points: 0,
                experience_level: 1,
                mana_points: 55,
                magic_level: 0,
                ammunition: 1,
            },
            outfit: OutfitColors::new(0, 0, 0, 0),
            gender: Gender::Male,
        }
    })
}

pub fn create_player(name: &str) -> Player {
    load_player_by_name(name).unwrap()
}
