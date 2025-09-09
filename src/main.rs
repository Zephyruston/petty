mod pet;
mod state;
mod ui;

use anyhow::Result;
use chrono::Utc;
use pet::{Pet, PetStatus};
use state::{delete_state, load_state, save_state};
use std::env;
use ui::run_ui;

const ABANDON_SECONDS: i64 = 3 * 24 * 60 * 60; // 3 days

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments for pet name
    let args: Vec<String> = env::args().collect();
    let pet_name = if args.len() > 1 {
        args[1].clone()
    } else {
        "Petty".to_string()
    };

    let mut pet = load_state().unwrap_or_else(|_| Pet::new(pet_name));

    // Check for abandonment and calculate elapsed time effects
    let now = Utc::now();
    let duration_since_last_seen = now.signed_duration_since(pet.last_seen);
    let elapsed_seconds = duration_since_last_seen.num_seconds();

    if elapsed_seconds > ABANDON_SECONDS {
        pet.status = PetStatus::Abandoned;
    } else if pet.status == PetStatus::Alive && pet.health > 0 {
        // Apply state changes for elapsed time (when pet is alive and not dead)
        apply_elapsed_time_effects(&mut pet, elapsed_seconds);
    }

    run_ui(&mut pet).await?;

    // Handle post-run state
    if pet.status == PetStatus::Abandoned || pet.health == 0 {
        // If pet was abandoned or died, delete the state to start fresh next time
        delete_state()?;
    } else {
        // Otherwise, update last_seen and save
        pet.last_seen = Utc::now();
        save_state(&pet)?;
    }

    Ok(())
}

/// Apply state changes based on elapsed time
fn apply_elapsed_time_effects(pet: &mut Pet, elapsed_seconds: i64) {
    // Calculate how many 3-second intervals have passed
    let intervals = elapsed_seconds / 3;

    for _ in 0..intervals {
        // Apply the same state changes as in the UI loop
        pet.hunger = pet.hunger.saturating_add(2);
        pet.cleanliness = pet.cleanliness.saturating_sub(3);

        // Health decreases if stats are poor
        // More nuanced health decline based on severity
        if pet.hunger > 90 {
            pet.health = pet.health.saturating_sub(3);
        } else if pet.hunger > 80 {
            pet.health = pet.health.saturating_sub(2);
        } else if pet.hunger > 70 {
            pet.health = pet.health.saturating_sub(1);
        }

        if pet.cleanliness < 10 {
            pet.health = pet.health.saturating_sub(3);
        } else if pet.cleanliness < 20 {
            pet.health = pet.health.saturating_sub(2);
        } else if pet.cleanliness < 30 {
            pet.health = pet.health.saturating_sub(1);
        }

        if pet.mood < 10 {
            pet.health = pet.health.saturating_sub(3);
        } else if pet.mood < 20 {
            pet.health = pet.health.saturating_sub(2);
        } else if pet.mood < 30 {
            pet.health = pet.health.saturating_sub(1);
        }

        // Age affects health decline - older pets decline faster
        if pet.age > 50 {
            // Elderly pet - health declines faster
            if pet.hunger > 60 || pet.cleanliness < 40 || pet.mood < 40 {
                pet.health = pet.health.saturating_sub(1);
            }
        } else if pet.age > 20 {
            // Adult pet - normal health decline
            // No additional effect
        }

        // Check for sickness when health is low
        if pet.health < 20 && pet.status == PetStatus::Alive {
            pet.status = PetStatus::Sick;
        }

        // Check for death when health is extremely low
        if pet.health == 0 {
            break; // Stop processing if pet dies
        }
    }

    // Apply aging (every 5 minutes = 300 seconds)
    let aging_periods = elapsed_seconds / 300;
    pet.age = pet.age.saturating_add(aging_periods as u32);

    // Apply mood decline (every second)
    let mood_decline = elapsed_seconds * 2; // Mood drops faster
    pet.mood = pet.mood.saturating_sub(mood_decline as u8);

    // Apply sleep healing if sleeping
    if pet.is_sleeping {
        let healing = elapsed_seconds; // 1 health point per second while sleeping
        pet.health = pet.health.saturating_add(healing as u8);

        // Elderly pets heal slower
        if pet.age > 50 {
            pet.health = pet.health.saturating_sub(healing as u8);
        }
    }
}
