use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use crate::core::{Direction, GameState, UserAction};
use crate::models::Cell::{
    BoxOnFloor, BoxOnTarget, Floor, PlayerOnFloor, PlayerOnTarget, Target, Wall,
};
use crate::models::{Cell, GameRenderState, Vec2};

pub fn parse_level(s: &str) -> GameState {
    let mut grid: Vec<Vec<Cell>> = Vec::new();
    let mut player = Vec2 { i: 0, j: 0 };
    let max_width = s.lines().map(|line| line.len()).max().unwrap_or(0);
    for (i, line) in s.lines().enumerate() {
        let mut row = Vec::new();
        for (j, ch) in line.chars().enumerate() {
            let c = match ch {
                '#' => Wall,
                ' ' => Floor,
                '.' => Target,
                '$' => BoxOnFloor,
                '*' => BoxOnTarget,
                '@' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    PlayerOnFloor
                }
                '+' => {
                    player = Vec2 {
                        i: i as i32,
                        j: j as i32,
                    };
                    PlayerOnTarget
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
    }

    GameState {
        grid,
        player,
    }
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
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
    state: &GameRenderState
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        // Game area
        let game_text = render_grid_to_string(&state.game.grid);
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

        let instruction_paragraph = Paragraph::new(instructions)
            .block(Block::default().borders(Borders::ALL).title("Instructions"))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(instruction_paragraph, chunks[1]);
    })?;
    Ok(())
}

fn render_grid_to_string(grid: &Vec<Vec<Cell>>) -> String {
    let mut result = String::new();
    for row in grid {
        for c in row {
            let ch = match c {
                Wall => '#',
                Floor => ' ',
                Target => '.',
                BoxOnFloor => '$',
                BoxOnTarget => '*',
                PlayerOnFloor => '@',
                PlayerOnTarget => '+',
            };
            result.push(ch);
        }
        result.push('\n');
    }
    result
}

pub fn handle_input() -> Result<Option<UserAction>, Box<dyn std::error::Error>> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return Ok(Some(UserAction::Quit));
                }
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    return Ok(Some(UserAction::Move(Direction::Up)));
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    return Ok(Some(UserAction::Move(Direction::Down)));
                }
                KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                    return Ok(Some(UserAction::Move(Direction::Left)));
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                    return Ok(Some(UserAction::Move(Direction::Right)));
                }
                _ => {}
            }
        }
    }
    Ok(None)
}
