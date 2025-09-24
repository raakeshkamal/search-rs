//! Event handling for keyboard and mouse input

use crate::{Result, SearchError};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::time::Duration;

/// Event handler for TUI input
pub struct EventHandler;

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Poll for the next event with timeout
    pub fn next_event(&self, timeout: Duration) -> Result<Option<Event>> {
        if event::poll(timeout)
            .map_err(|e| SearchError::TuiError(format!("Event polling failed: {}", e)))?
        {
            let event = event::read()
                .map_err(|e| SearchError::TuiError(format!("Event reading failed: {}", e)))?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    /// Handle a mouse event and return the action to take
    pub fn handle_mouse_event(&self, event: MouseEvent) -> MouseAction {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                MouseAction::ClickAt(event.column, event.row)
            }
            _ => MouseAction::None,
        }
    }

    /// Handle a key event and return the action to take
    pub fn handle_key_event(&self, event: KeyEvent) -> KeyAction {
        match event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => KeyAction::Quit,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => KeyAction::Quit,
            KeyEvent {
                code: KeyCode::Up, ..
            } => KeyAction::MovePrevious,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => KeyAction::MoveNext,
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => KeyAction::OpenFile,
            KeyEvent {
                code: KeyCode::Tab, ..
            } => KeyAction::CycleFocus,
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => KeyAction::RefreshSearch,
            KeyEvent {
                code: KeyCode::Char('/'),
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => KeyAction::FocusSearch,
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::InputChar(c),
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            } => KeyAction::DeleteChar,
            _ => KeyAction::None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum KeyAction {
    Quit,
    MovePrevious,
    MoveNext,
    OpenFile,
    CycleFocus,
    RefreshSearch,
    FocusSearch,
    InputChar(char),
    DeleteChar,
    None,
}

#[derive(Debug, PartialEq)]
pub enum MouseAction {
    None,
    ClickAt(u16, u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler() -> EventHandler {
        EventHandler::new().unwrap()
    }

    fn assert_key_action(key_code: KeyCode, modifiers: KeyModifiers, expected: KeyAction) {
        let handler = test_handler();
        let event = KeyEvent::new(key_code, modifiers);
        assert_eq!(handler.handle_key_event(event), expected);
    }

    fn create_mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: column,
            row: row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn test_key_mappings_data_driven() {
        let test_cases = [
            (KeyCode::Esc, KeyModifiers::NONE, KeyAction::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL, KeyAction::Quit),
            (KeyCode::Up, KeyModifiers::NONE, KeyAction::MovePrevious),
            (KeyCode::Down, KeyModifiers::NONE, KeyAction::MoveNext),
            (KeyCode::Enter, KeyModifiers::NONE, KeyAction::OpenFile),
            (KeyCode::Tab, KeyModifiers::NONE, KeyAction::CycleFocus),
            (
                KeyCode::Char('r'),
                KeyModifiers::CONTROL,
                KeyAction::RefreshSearch,
            ),
            (
                KeyCode::Char('/'),
                KeyModifiers::NONE,
                KeyAction::FocusSearch,
            ),
            (
                KeyCode::Char('f'),
                KeyModifiers::CONTROL,
                KeyAction::FocusSearch,
            ),
            (
                KeyCode::Char('a'),
                KeyModifiers::NONE,
                KeyAction::InputChar('a'),
            ),
            (
                KeyCode::Backspace,
                KeyModifiers::NONE,
                KeyAction::DeleteChar,
            ),
            (
                KeyCode::Char('j'),
                KeyModifiers::NONE,
                KeyAction::InputChar('j'),
            ),
            (
                KeyCode::Char('k'),
                KeyModifiers::NONE,
                KeyAction::InputChar('k'),
            ),
            (KeyCode::F(1), KeyModifiers::NONE, KeyAction::None),
            (KeyCode::Char('a'), KeyModifiers::ALT, KeyAction::None),
        ];

        for (key_code, modifiers, expected) in test_cases {
            assert_key_action(key_code, modifiers, expected);
        }

        let chars = ['a', 'Z', '1', '@', ' ', '-', '_'];
        for c in chars {
            assert_key_action(
                KeyCode::Char(c),
                KeyModifiers::NONE,
                KeyAction::InputChar(c),
            );
        }
    }

    #[test]
    fn test_key_action_debug_trait() {
        let cases = [
            (KeyAction::Quit, "Quit"),
            (KeyAction::MovePrevious, "MovePrevious"),
            (KeyAction::MoveNext, "MoveNext"),
            (KeyAction::OpenFile, "OpenFile"),
            (KeyAction::CycleFocus, "CycleFocus"),
            (KeyAction::RefreshSearch, "RefreshSearch"),
            (KeyAction::FocusSearch, "FocusSearch"),
            (KeyAction::DeleteChar, "DeleteChar"),
            (KeyAction::None, "None"),
        ];

        for (action, expected) in cases {
            assert_eq!(format!("{:?}", action), expected);
        }

        assert_eq!(format!("{:?}", KeyAction::InputChar('a')), "InputChar('a')");
    }

    #[test]
    fn test_key_action_partial_eq_trait() {
        // Test InputChar equality
        assert_eq!(KeyAction::InputChar('a'), KeyAction::InputChar('a'));
        assert_eq!(KeyAction::InputChar('z'), KeyAction::InputChar('z'));

        // Test inequalities between different variants
        assert_ne!(KeyAction::Quit, KeyAction::MovePrevious);
        assert_ne!(KeyAction::OpenFile, KeyAction::None);
        assert_ne!(KeyAction::RefreshSearch, KeyAction::FocusSearch);

        // Test InputChar inequality
        assert_ne!(KeyAction::InputChar('a'), KeyAction::InputChar('b'));
        assert_ne!(KeyAction::InputChar('a'), KeyAction::Quit);
    }

    #[test]
    fn test_mouse_event_handler() {
        let handler = test_handler();

        // Positive case: Left button down
        let event = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 1, 2);
        assert_eq!(
            handler.handle_mouse_event(event),
            MouseAction::ClickAt(1, 2)
        );

        // Negative cases: other events and buttons
        let negative_kinds = [
            MouseEventKind::Down(MouseButton::Right),
            MouseEventKind::Down(MouseButton::Middle),
            MouseEventKind::Up(MouseButton::Left),
            MouseEventKind::Drag(MouseButton::Left),
            MouseEventKind::Moved,
            MouseEventKind::ScrollUp,
            MouseEventKind::ScrollDown,
        ];

        for kind in negative_kinds {
            let event = create_mouse_event(kind, 0, 0);
            assert_eq!(
                handler.handle_mouse_event(event),
                MouseAction::None,
                "Failed for kind: {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_mouse_action_partial_eq_trait() {
        // Test None equality
        assert_eq!(MouseAction::None, MouseAction::None);

        assert_eq!(
            MouseAction::ClickAt(1u16, 20u16),
            MouseAction::ClickAt(1u16, 20u16)
        );

        // Test inequality between None and ClickAt
        assert_ne!(MouseAction::None, MouseAction::ClickAt(1, 1));

        // Test inequality between different ClickAt
        assert_ne!(MouseAction::ClickAt(1, 1), MouseAction::ClickAt(2, 2));
    }
}
