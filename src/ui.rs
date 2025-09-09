use crate::pet::{Pet, PetStatus};
use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures_util::StreamExt as FuturesStreamExt;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::stdout;
use std::process;
use std::time::Duration;
use tokio::time::interval;

fn suspend_and_restore() {
    // Exit alternate screen and raw mode
    let mut stdout = stdout();
    stdout.execute(LeaveAlternateScreen).ok();
    disable_raw_mode().ok();

    unsafe {
        // Restore default SIGTSTP behavior
        libc::signal(libc::SIGTSTP, libc::SIG_DFL);
        // Send SIGTSTP to self
        libc::kill(process::id() as i32, libc::SIGTSTP);
    }

    // When user uses `fg` to resume, re-enter raw mode and alternate screen
    enable_raw_mode().ok();
    stdout.execute(EnterAlternateScreen).ok();
}

pub async fn run_ui(pet: &mut Pet) -> Result<()> {
    // setup terminal
    crossterm::terminal::enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut ticker = interval(Duration::from_secs(1));
    let mut seconds_elapsed = 0;
    let mut event_stream = futures_util::StreamExt::fuse(event::EventStream::new());
    let mut input_buffer = String::new();

    loop {
        terminal.draw(|f| ui(f, pet))?;

        if pet.status == PetStatus::Abandoned {
            // If abandoned, only allow quitting
            if let Some(Ok(Event::Key(key))) = event_stream.next().await
                && (key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL))
            {
                break;
            }
            continue;
        }

        tokio::select! {
            _ = ticker.tick() => {
                seconds_elapsed += 1;

                if pet.debug_mode {
                    continue; // Pause state changes in debug mode
                }

                // Age the pet every 5 minutes (300 seconds)
                if seconds_elapsed > 0 && seconds_elapsed % 300 == 0 {
                    pet.age = pet.age.saturating_add(1);
                }

                if pet.is_sleeping {
                    pet.health = pet.health.saturating_add(1);
                    // Elderly pets heal slower
                    if pet.life_stage() == "elderly" {
                        pet.health = pet.health.saturating_sub(1);
                    }
                } else {
                    if pet.mood > 0 {
                        pet.mood = pet.mood.saturating_sub(2); // Mood drops faster
                    }
                    if seconds_elapsed % 3 == 0 { // Status changes every 3 seconds
                        pet.hunger = pet.hunger.saturating_add(2);
                        pet.cleanliness = pet.cleanliness.saturating_sub(3);

                        // Health decreases if stats are poor
                        if pet.hunger > 80 {
                            pet.health = pet.health.saturating_sub(1);
                        }
                        if pet.cleanliness < 20 {
                            pet.health = pet.health.saturating_sub(1);
                        }
                        if pet.mood < 20 {
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
                    }
                }
            },

            event = event_stream.select_next_some() => {
                if let Ok(Event::Key(key)) = event
                    && key.kind == KeyEventKind::Press {
                        // Always allow exit
                        if key.code == KeyCode::Char('q') || (key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL) {
                            break;
                        }

                        // Handle Ctrl+Z
                        if key.code == KeyCode::Char('z') && key.modifiers == KeyModifiers::CONTROL {
                            suspend_and_restore();
                            // Recreate terminal after resume
                            terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
                            continue;
                        }

                        if pet.debug_mode {
                            match key.code {
                                KeyCode::Char('h') => pet.hunger = pet.hunger.saturating_add(10),
                                KeyCode::Char('j') => pet.hunger = pet.hunger.saturating_sub(10),
                                KeyCode::Char('m') => pet.mood = pet.mood.saturating_add(10),
                                KeyCode::Char('n') => pet.mood = pet.mood.saturating_sub(10),
                                KeyCode::Char('c') => pet.cleanliness = pet.cleanliness.saturating_add(10),
                                KeyCode::Char('v') => pet.cleanliness = pet.cleanliness.saturating_sub(10),
                                KeyCode::Esc => {
                                    pet.debug_mode = false;
                                    input_buffer.clear();
                                }
                                _ => {}
                            }
                        } else {
                            if let KeyCode::Char(c) = key.code {
                                input_buffer.push(c);
                                if input_buffer.ends_with("debug") {
                                    pet.debug_mode = true;
                                    input_buffer.clear();
                                }
                            } else {
                                input_buffer.clear();
                            }

                            if pet.is_sleeping && key.code != KeyCode::Char('s') {
                                continue;
                            }
                            match key.code {
                                KeyCode::Char('f') => {
                                    pet.feed();
                                    // Elderly pets get less benefit from feeding
                                    if pet.life_stage() == "elderly" {
                                        pet.health = pet.health.saturating_sub(2);
                                    }
                                },
                                KeyCode::Char('w') => {
                                    pet.wash();
                                    // Elderly pets get stressed from washing
                                    if pet.life_stage() == "elderly" {
                                        pet.mood = pet.mood.saturating_sub(5);
                                    }
                                },
                                KeyCode::Char('p') => {
                                    pet.play();
                                    // Elderly pets get tired more easily
                                    if pet.life_stage() == "elderly" {
                                        pet.health = pet.health.saturating_sub(3);
                                    }
                                },
                                KeyCode::Char('s') => pet.sleep(),
                                _ => {}
                            }
                        }
                    }
            }
        }
    }

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn ui(frame: &mut Frame, pet: &Pet) {
    if pet.status == PetStatus::Abandoned {
        let message = vec![
            Line::from(""),
            Line::from("你的宠物因为被忽视太久，离家出走了..."),
            Line::from(""),
            Line::from(Span::styled(
                "按 'q' 或 'ctrl-c' 退出，下次启动将开始新的旅程。",
                Style::default().add_modifier(Modifier::ITALIC),
            )),
        ];
        let paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, frame.area());
        return;
    }

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Hint bar
        ])
        .split(frame.area());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(main_layout[0]);

    let pet_art_lines = if pet.debug_mode {
        vec![
            "",
            r"     /\_/\",
            r"     ( o_o )",
            r"     > ^ <",
            "别再戳我了，我在休假！",
        ]
    } else if pet.is_sleeping {
        vec![
            "",
            r"      /\_/\",
            r"           ( - . - ) Zzz",
            r"      > ^ <",
            "",
        ]
    } else {
        match pet.life_stage() {
            "elderly" => {
                // Elderly pet
                vec!["", r"     /\_/\", r"     ( -_- )", r"     > v <", ""]
            }
            "adult" => {
                // Adult pet
                vec!["", r"     /\_/\", r"     ( ._. )", r"     > ^ <", ""]
            }
            _ => {
                // Young pet
                if pet.mood < 20 {
                    // Sad pet
                    vec!["", r"     /\_/\", r"     ( T.T )", r"     > ^ <", ""]
                } else if pet.hunger > 60 {
                    // Hungry pet
                    vec!["", r"     /\_/\", r"     ( o_o )", r"     > ^ <", ""]
                } else if pet.cleanliness < 40 {
                    // Dirty pet
                    vec!["", r"     /\_/\", r"     ( >.< )", r"     > ^ <", ""]
                } else if pet.mood > 80 {
                    // Happy pet
                    vec!["", r"     /\_/\", r"     ( ^.^ )", r"     > ^ <", ""]
                } else {
                    // Neutral young pet
                    vec!["", r"     /\_/\", r"     ( o.o )", r"     > ^ <", ""]
                }
            }
        }
    };
    let pet_art = Paragraph::new(pet_art_lines.join("\n")).alignment(Alignment::Center);

    let pet_view = Block::default().title("Pet").borders(Borders::ALL);
    frame.render_widget(pet_art.block(pet_view), top_layout[0]);

    let stats_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1); 6].as_ref())
        .split(top_layout[1]);

    let stats_view = Block::default().title("Stats").borders(Borders::ALL);

    let name = Paragraph::new(format!("Name: {}", pet.name));
    let age = Paragraph::new(format!("Age: {}", pet.age));
    let health = Paragraph::new(format!("Health: {}", pet.health));
    let hunger = Paragraph::new(format!("Hunger: {}", pet.hunger));
    let cleanliness = Paragraph::new(format!("Cleanliness: {}", pet.cleanliness));
    let mood = Paragraph::new(format!("Mood: {}", pet.mood));

    frame.render_widget(stats_view, top_layout[1]);
    frame.render_widget(name, stats_layout[0]);
    frame.render_widget(age, stats_layout[1]);
    frame.render_widget(health, stats_layout[2]);
    frame.render_widget(hunger, stats_layout[3]);
    frame.render_widget(cleanliness, stats_layout[4]);
    frame.render_widget(mood, stats_layout[5]);

    let hints = if pet.debug_mode {
        Paragraph::new(" [Debug Mode] (h/j) Hunger | (m/n) Mood | (c/v) Cleanliness | (Esc) Exit ")
            .alignment(Alignment::Center)
    } else {
        Paragraph::new(" (f)eed | (w)ash | (p)lay | (s)leep | (q)uit | ctrl-c | ctrl-z ")
            .alignment(Alignment::Center)
    };
    frame.render_widget(hints, main_layout[1]);
}
