use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;

#[allow(dead_code)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}
