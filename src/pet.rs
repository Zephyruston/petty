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

    pub fn life_stage(&self) -> &'static str {
        if self.age > 50 {
            "elderly"
        } else if self.age > 20 {
            "adult"
        } else {
            "young"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_creation() {
        let pet = Pet::new("TestPet".to_string());
        assert_eq!(pet.name, "TestPet");
        assert_eq!(pet.age, 0);
        assert_eq!(pet.health, 100);
    }

    #[test]
    fn test_life_stage() {
        let mut pet = Pet::new("TestPet".to_string());

        // Test young pet
        pet.age = 10;
        assert_eq!(pet.life_stage(), "young");

        // Test adult pet
        pet.age = 30;
        assert_eq!(pet.life_stage(), "adult");

        // Test elderly pet
        pet.age = 60;
        assert_eq!(pet.life_stage(), "elderly");
    }

    #[test]
    fn test_feed() {
        let mut pet = Pet::new("TestPet".to_string());
        pet.hunger = 50;
        pet.feed();
        assert_eq!(pet.hunger, 30);
    }

    #[test]
    fn test_wash() {
        let mut pet = Pet::new("TestPet".to_string());
        pet.cleanliness = 50;
        pet.wash();
        assert_eq!(pet.cleanliness, 100);
    }

    #[test]
    fn test_play() {
        let mut pet = Pet::new("TestPet".to_string());
        let initial_hunger = pet.hunger;
        let initial_mood = pet.mood;
        pet.play();
        assert_eq!(pet.mood, initial_mood + 10);
        assert_eq!(pet.hunger, initial_hunger + 5);
    }

    #[test]
    fn test_sleep() {
        let mut pet = Pet::new("TestPet".to_string());
        assert!(!pet.is_sleeping);
        pet.sleep();
        assert!(pet.is_sleeping);
        pet.sleep();
        assert!(!pet.is_sleeping);
    }
}
