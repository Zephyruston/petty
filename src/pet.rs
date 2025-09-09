use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum PetStatus {
    Alive,
    Abandoned,
}

#[derive(Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub age: u32,
    pub health: u8,
    pub hunger: u8,
    pub cleanliness: u8,
    pub mood: u8,
    pub is_sleeping: bool,
    pub status: PetStatus,
    pub last_seen: DateTime<Utc>,
    pub debug_mode: bool,
}

impl Pet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            age: 0,
            health: 100,
            hunger: 0,
            cleanliness: 100,
            mood: 100,
            is_sleeping: false,
            status: PetStatus::Alive,
            last_seen: Utc::now(),
            debug_mode: false,
        }
    }

    pub fn feed(&mut self) {
        self.hunger = self.hunger.saturating_sub(20);
        self.health = self.health.saturating_add(5);
    }

    pub fn wash(&mut self) {
        self.cleanliness = 100;
    }

    pub fn play(&mut self) {
        self.mood = self.mood.saturating_add(10);
        self.hunger = self.hunger.saturating_add(5);
    }

    pub fn sleep(&mut self) {
        self.is_sleeping = !self.is_sleeping;
    }
}
