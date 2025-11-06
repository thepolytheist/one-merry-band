//! One Merry Band
//!
//! A Tolkienesque roguelite where the player controls a band of adventurers
//! exploring the depths of the Romia Mines.

mod components;
mod game;
mod level;
mod systems;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::components::AdventurerType;
use crate::game::{Game, GameState};

/// Party selection state
struct PartySelection {
    selected_adventurers: Vec<(AdventurerType, usize)>,
    cursor: usize,
    points_remaining: i32,
}

impl PartySelection {
    fn new() -> Self {
        Self {
            selected_adventurers: Vec::new(),
            cursor: 0,
            points_remaining: 10,
        }
    }

    /// Handle input during party selection
    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<AdventurerType>> {
        let adventurer_types = [
            AdventurerType::Fighter,
            AdventurerType::Archer,
            AdventurerType::Wizard,
            AdventurerType::Scout,
        ];

        match key.code {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor < adventurer_types.len() - 1 {
                    self.cursor += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let adventurer_type = adventurer_types[self.cursor];
                let cost = adventurer_type.cost();

                if self.points_remaining >= cost {
                    self.selected_adventurers.push((adventurer_type, self.selected_adventurers.len()));
                    self.points_remaining -= cost;
                }
            }
            KeyCode::Backspace => {
                if let Some((adventurer_type, _)) = self.selected_adventurers.pop() {
                    self.points_remaining += adventurer_type.cost();
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Confirm selection
                if !self.selected_adventurers.is_empty() {
                    return Some(
                        self.selected_adventurers
                            .iter()
                            .map(|(t, _)| *t)
                            .collect(),
                    );
                }
            }
            _ => {}
        }

        None
    }

    fn get_selected_list(&self) -> &[(AdventurerType, usize)] {
        &self.selected_adventurers
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the game
    let res = run_game(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_game<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut game = Game::new();
    let mut party_selection = PartySelection::new();

    loop {
        match game.state {
            GameState::PartySelection => {
                // Render party selection
                terminal.draw(|f| {
                    ui::render_party_selection(
                        f,
                        party_selection.get_selected_list(),
                        party_selection.cursor,
                        party_selection.points_remaining,
                    );
                })?;

                // Handle input
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }

                        if let Some(adventurers) = party_selection.handle_input(key) {
                            game.start_game(adventurers);
                        }
                    }
                }
            }
            GameState::Playing => {
                // Process AI turns first
                game.process_ai_turns();

                // Render game
                terminal.draw(|f| {
                    ui::render_game(f, &game.world);
                })?;

                // Handle input
                if let Some(key) = game::poll_input(Duration::from_millis(50))? {
                    if game.handle_input(key)? {
                        return Ok(());
                    }
                }
            }
            GameState::GameOver => {
                terminal.draw(|f| {
                    ui::render_game(f, &game.world);
                })?;

                // Wait for input to exit
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(_) = event::read()? {
                        return Ok(());
                    }
                }
            }
            GameState::Victory => {
                terminal.draw(|f| {
                    ui::render_game(f, &game.world);
                })?;

                // Wait for input to exit
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(_) = event::read()? {
                        return Ok(());
                    }
                }
            }
        }
    }
}
