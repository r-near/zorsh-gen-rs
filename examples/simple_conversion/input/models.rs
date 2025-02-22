use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Item {
    id: u32,
    name: String,
    attributes: HashMap<String, u32>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Inventory {
    items: Vec<Item>,
    capacity: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum PlayerStatus {
    Online,
    Offline { last_seen: u64 },
    Away(String),
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Player {
    name: String,
    level: u8,
    experience: u32,
    inventory: Inventory,
    status: Option<PlayerStatus>,
    achievements: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct GameState {
    players: HashMap<String, Player>,
    current_round: u32,
    timestamp: u64,
}
