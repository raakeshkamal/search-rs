//! UI rendering and layout module

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Information about the results list area for mouse click handling
#[derive(Debug, Clone)]
pub struct ResultsAreaInfo {
    pub top: u16,
    pub height: u16,
    pub left: u16,
    pub width: u16,
}

