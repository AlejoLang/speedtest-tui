use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame};
use crate::{ping_component::{self, PingComponent}, servers::Servers, tcp_tests::TcpTestLatency};
use crate::tcp_tests::TcpTest;
use tokio::{net::tcp, sync::mpsc};

#[derive(Default)]
pub struct App {
    running: bool,
    servers: Servers,
    ping_component: PingComponent,
}

impl App {
    pub fn new() -> Self {
        let ping_component = PingComponent::new(TcpTest::default());
        Self {
            running: true,
            servers: Servers::default(),
            ping_component,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        let server_update_result = self.servers.update_servers().await;
        match server_update_result {
            Ok(_) => {
            }
            Err(e) => {
                eprintln!("Failed to update servers: {}", e);
                return Err(e.into());
            }
        }
        let current_server = self.servers.get_servers()[0].clone(); 
        self.ping_component.set_server(current_server);

        while self.running {
            self.ping_component.check_latency_results().await;
            terminal.draw(|frame| self.render(frame))?;
            
            self.handle_crossterm_events_async().await?;
            
            tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&self.ping_component, frame.area());
    }

    async fn handle_crossterm_events_async(&mut self) -> Result<()> {
        // Usar poll para verificar si hay eventos disponibles sin bloquear
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    async fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            
            // Medir latencia (no bloqueante)
            (_, KeyCode::Enter) => {
                self.ping_component.start_latency_measurement();
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}