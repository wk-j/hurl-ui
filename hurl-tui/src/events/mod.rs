//! Event handling module
//!
//! This module provides event handling for terminal input events.

use anyhow::Result;
use crossterm::event::{self, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Terminal event types
#[derive(Debug, Clone)]
pub enum Event {
    /// Periodic tick event
    Tick,
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),
}

/// Event handler that polls for terminal events
pub struct EventHandler {
    /// Event receiver
    rx: mpsc::UnboundedReceiver<Event>,
    /// Tick rate in milliseconds
    _tick_rate: u64,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate (in milliseconds)
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tick_rate_duration = Duration::from_millis(tick_rate);

        // Spawn tick task - sends tick events at regular intervals
        let tick_tx = tx.clone();
        tokio::spawn(async move {
            let mut tick_interval = interval(tick_rate_duration);
            loop {
                tick_interval.tick().await;
                if tick_tx.send(Event::Tick).is_err() {
                    break;
                }
            }
        });

        // Spawn event polling task - handles input events
        tokio::spawn(async move {
            loop {
                // Poll for events with a short timeout to remain responsive
                if event::poll(Duration::from_millis(1)).unwrap_or(false) {
                    match event::read() {
                        Ok(event::Event::Key(key)) => {
                            if tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(event::Event::Mouse(mouse)) => {
                            if tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(event::Event::Resize(width, height)) => {
                            if tx.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                } else {
                    // Yield to prevent busy-spinning when no events
                    tokio::task::yield_now().await;
                }
            }
        });

        Self {
            rx,
            _tick_rate: tick_rate,
        }
    }

    /// Get the next event
    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
