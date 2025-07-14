use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{widgets::{Block, Paragraph}, DefaultTerminal, Frame};
use tokio::sync::mpsc;
use crate::{http_tester::HttpTester, ping_component::PingComponent, servers::Servers, services::HttpTestService};

pub struct App {
    running: bool,
    servers: Servers,
    test_service: HttpTestService,
    ping_component: PingComponent,
    testing: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            servers: Servers::default(),
            test_service: HttpTestService::new(HttpTester::default()),
            ping_component: PingComponent::default(),
            testing: false
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
        let current_server = self.servers.get_servers()[5].clone(); 
        let url = format!("http://{}", current_server.host);
        self.test_service.set_tester(HttpTester::new(url.as_str()));

        while self.running {
            self.test_service.check_measurments().await;

            if !self.test_service.get_testing() {
                let new_ping_measurment = self.test_service.get_ping_results().clone();
                self.ping_component.set_ping_measurement(new_ping_measurment);
            }

            terminal.draw(|frame| self.render(frame))?;
            
            self.handle_crossterm_events();
            
            tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&self.ping_component, frame.area());
        let url = self.servers.get_servers()[0].host.clone();
        let p = Block::default().title(url.as_str()).borders(ratatui::widgets::Borders::ALL);
        frame.render_widget(p, frame.area());
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        // Usar poll para verificar si hay eventos disponibles sin bloquear
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            
            // Medir latencia (no bloqueante)
            (_, KeyCode::Enter) => {
                self.test_service.run_full_test();
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}