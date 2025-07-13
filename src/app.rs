use std::{thread::sleep, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{style::Stylize, text::{Line, Span, Text}, widgets::{Block, Paragraph}, DefaultTerminal, Frame};
use crate::servers::Servers;

#[derive(Default)]
pub struct App {
    running: bool,
    servers: Servers
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        self.servers.update_servers().await;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let mut servers_lines = Vec::new();
        for server in self.servers.get_servers() {
            let line = Line::from(format!("{}, {}", server.name, server.sponsor));
            servers_lines.push(line);
        }
        let servers_text = Text::from(servers_lines);
        frame.render_widget(
            Paragraph::new(servers_text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}