use crate::core::{Direction, GameState, SharedGameState, UserAction};
use crate::models::Cell::{
    Floor, Target, Wall,
};
use crate::models::{Cell, GameRenderState, Vec2};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::io;

pub fn parse_level(s: &str) -> (GameState, SharedGameState) {
    let mut grid: Vec<Vec<Cell>> = Vec::new();
    let mut player = Vec2 { i: 0, j: 0 };
    let mut boxes: Vec<Vec2> = Vec::new();
    let max_width = s.lines().map(|line| line.len()).max().unwrap_or(0);

    let mut i = 0;
    for line in s.lines() {
        let line = line.trim_matches('\n');
        if line.len() == 0 {
            continue;
        }

        let mut row = Vec::new();
        for (j, ch) in line.chars().enumerate() {
            let c = match ch {
                '#' => Wall,
                ' ' => Floor,
                '.' => Target,
                '$' => {
                    boxes.push(Vec2 {
                        i: i as i32,
                        j: j as i32,
                    });
                    Floor
                },
                '*' => {
                    boxes.push(Vec2 {
                        i: i as i32,
                        j: j as i32,
                    });
                    Target
                },
                '@' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    Floor
                }
                '+' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    Target
                }
                _ => Floor,
            };
            row.push(c);
        }
        // Pad row to max width with Floor
        while row.len() < max_width {
            row.push(Floor);
        }
        grid.push(row);
        i += 1;
    }

    (
        GameState {
            player,
            boxes
        },
        SharedGameState {
            grid,
        }
    )
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>>
{
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn cleanup_terminal() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

pub fn render_game(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    shared: &SharedGameState,
    state: &GameRenderState,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        // Game area
        let game_text = render_game_to_string(shared, &state.game);
        let game_paragraph = Paragraph::new(game_text)
            .block(Block::default().borders(Borders::ALL).title("Sokoban"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(game_paragraph, chunks[0]);

        // Instructions
        let instructions = if state.won {
            "🎉 You Win! Press the any key to quit."
        } else {
            "Controls: WASD or Arrow keys to move, Q to quit"
        };

        let instructions = if let Some(err) = &state.error {
            format!("{} | Error: {}", instructions, err)
        } else {
            instructions.to_string()
        };

        let instructions = if let Some(change_type) = &state.last_change {
            format!("{} | Last: {:?}", instructions, change_type)
        } else {
            instructions
        };

        let instruction_paragraph = Paragraph::new(instructions)
            .block(Block::default().borders(Borders::ALL).title("Instructions"))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(instruction_paragraph, chunks[1]);
    })?;
    Ok(())
}

pub fn render_game_to_string(shared: &SharedGameState, game: &GameState) -> String {
    let mut result = String::new();
    for (i, row) in shared.grid.iter().enumerate() {
        for (j, c) in row.iter().enumerate() {
            let pos = Vec2 {
                i: i as i32,
                j: j as i32,
            };
            let has_player = pos == game.player;
            let has_box = game.boxes.contains(&pos);
            let ch = match c {
                Wall => '#',
                Floor => if has_player { '@' } else { if has_box { '$' } else { ' ' } },
                Target => if has_player { '+' } else { if has_box { '*' } else { '.' } },
            };
            result.push(ch);
        }
        result.push('\n');
    }
    result
}

pub enum ConsoleInput {
    UserAction(UserAction),
    Quit,
    Timeout,
    Unknown,
}

pub fn handle_input() -> Result<ConsoleInput, Box<dyn std::error::Error>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => ConsoleInput::Quit,
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    ConsoleInput::UserAction(UserAction::Move(Direction::Up))
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    ConsoleInput::UserAction(UserAction::Move(Direction::Down))
                }
                KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                    ConsoleInput::UserAction(UserAction::Move(Direction::Left))
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                    ConsoleInput::UserAction(UserAction::Move(Direction::Right))
                }
                _ => ConsoleInput::Unknown,
            });
        }
    }
    Ok(ConsoleInput::Timeout)
}
