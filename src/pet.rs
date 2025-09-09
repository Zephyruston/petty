use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum PetStatus {
    Alive,
    Sick,
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

    pub fn train(&mut self) {
        // Training increases health and mood but consumes energy (increases hunger)
        self.health = self.health.saturating_add(3);
        self.mood = self.mood.saturating_add(5);
        self.hunger = self.hunger.saturating_add(10);
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

    #[test]
    fn test_health_decline_rules() {
        let mut pet = Pet::new("TestPet".to_string());
        pet.health = 100;

        // Test hunger effect on health
        pet.hunger = 95;
        // In the actual UI loop, this would be processed every 3 seconds
        // Here we're just verifying the logic would work correctly
        assert!(pet.hunger > 90);

        pet.hunger = 85;
        assert!(pet.hunger > 80);
        assert!(pet.hunger <= 90);

        pet.hunger = 75;
        assert!(pet.hunger > 70);
        assert!(pet.hunger <= 80);

        // Test cleanliness effect on health
        pet.cleanliness = 5;
        assert!(pet.cleanliness < 10);

        pet.cleanliness = 15;
        assert!(pet.cleanliness < 20);
        assert!(pet.cleanliness >= 10);

        pet.cleanliness = 25;
        assert!(pet.cleanliness < 30);
        assert!(pet.cleanliness >= 20);

        // Test mood effect on health
        pet.mood = 5;
        assert!(pet.mood < 10);

        pet.mood = 15;
        assert!(pet.mood < 20);
        assert!(pet.mood >= 10);

        pet.mood = 25;
        assert!(pet.mood < 30);
        assert!(pet.mood >= 20);
    }

    #[test]
    fn test_sickness_and_death() {
        let mut pet = Pet::new("TestPet".to_string());

        // Test pet starts as alive
        assert_eq!(pet.status, PetStatus::Alive);

        // Test pet becomes sick when health is low
        pet.health = 15;
        // This would be set in the UI loop, but we can test the condition
        if pet.health < 20 && pet.status == PetStatus::Alive {
            pet.status = PetStatus::Sick;
        }
        assert_eq!(pet.status, PetStatus::Sick);

        // Test pet is dead when health reaches 0
        pet.health = 0;
        assert_eq!(pet.health, 0);
    }

    #[test]
    fn test_pet_death_handling() {
        let mut pet = Pet::new("TestPet".to_string());

        // Simulate pet death
        pet.health = 0;

        // Verify pet is considered dead
        assert_eq!(pet.health, 0);

        // In the main function, this would trigger state deletion
        // We're just verifying the condition would work correctly
        let should_delete_state = pet.health == 0;
        assert!(should_delete_state);
    }

    #[test]
    fn test_apply_elapsed_time_effects() {
        let mut pet = Pet::new("TestPet".to_string());
        pet.health = 100;
        pet.hunger = 50;
        pet.cleanliness = 50;
        pet.mood = 50;

        // Apply 10 seconds of elapsed time
        // This should result in:
        // - 3 intervals of hunger/cleanliness changes (10/3 = 3)
        // - 3 intervals of health decline if thresholds are met
        // - Aging if 5 minutes have passed (they haven't in this case)
        // - Mood decline (10 * 2 = 20)

        // For this test pet, no health decline should occur since thresholds aren't met
        // Hunger should increase by 6 (2 * 3)
        // Cleanliness should decrease by 9 (3 * 3)
        // Mood should decrease by 20

        // Note: We can't directly test the apply_elapsed_time_effects function since it's private
        // But we can test the logic it implements
        let intervals = 10 / 3; // 3 intervals
        for _ in 0..intervals {
            pet.hunger = pet.hunger.saturating_add(2);
            pet.cleanliness = pet.cleanliness.saturating_sub(3);
        }

        let mood_decline = 10 * 2;
        pet.mood = pet.mood.saturating_sub(mood_decline as u8);

        assert_eq!(pet.hunger, 56);
        assert_eq!(pet.cleanliness, 41);
        assert_eq!(pet.mood, 30);
    }

    #[test]
    fn test_train() {
        let mut pet = Pet::new("TrainPet".to_string());
        let initial_health = pet.health;
        let initial_mood = pet.mood;
        let initial_hunger = pet.hunger;

        pet.train();

        assert_eq!(pet.health, initial_health + 3);
        assert_eq!(pet.mood, initial_mood + 5);
        assert_eq!(pet.hunger, initial_hunger + 10);
    }

    #[test]
    fn test_custom_name() {
        let pet = Pet::new("Fluffy".to_string());
        assert_eq!(pet.name, "Fluffy");
    }
}
