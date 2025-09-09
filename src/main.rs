mod pet;
mod state;
mod ui;

use anyhow::Result;
use chrono::Utc;
use pet::{Pet, PetStatus};
use state::{delete_state, load_state, save_state};
use ui::run_ui;

const ABANDON_SECONDS: i64 = 3 * 24 * 60 * 60; // 3 days

#[tokio::main]
async fn main() -> Result<()> {
    let mut pet = load_state().unwrap_or_else(|_| Pet::new("Petty".to_string()));

    // Check for abandonment
    let now = Utc::now();
    let duration_since_last_seen = now.signed_duration_since(pet.last_seen);

    if duration_since_last_seen.num_seconds() > ABANDON_SECONDS {
        pet.status = PetStatus::Abandoned;
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
