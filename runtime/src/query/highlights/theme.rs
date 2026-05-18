//! Theme and color handling for syntax highlighting.

use super::capture_names;
use std::collections::HashMap;

/// Theme colors for syntax highlighting.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Colors for different highlight types.
    pub colors: HashMap<String, Color>,
    /// Default color for unhighlighted text.
    pub default_color: Color,
    /// Background color.
    pub background_color: Color,
}

/// Color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Color {
    /// Create an RGB color.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to a hex string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl Theme {
    /// Create a default dark theme.
    pub fn dark() -> Self {
        Self {
            colors: dark_colors(),
            default_color: Color::new(212, 212, 212),
            background_color: Color::new(30, 30, 30),
        }
    }

    /// Create a default light theme.
    pub fn light() -> Self {
        Self {
            colors: light_colors(),
            default_color: Color::new(0, 0, 0),
            background_color: Color::new(255, 255, 255),
        }
    }

    /// Get color for a highlight type.
    pub fn get_color(&self, highlight: &str) -> Color {
        self.colors
            .get(highlight)
            .copied()
            .unwrap_or(self.default_color)
    }
}

fn dark_colors() -> HashMap<String, Color> {
    HashMap::from([
        color(capture_names::COMMENT, 106, 153, 85),
        color(capture_names::STRING, 206, 145, 120),
        color(capture_names::NUMBER, 181, 206, 168),
        color(capture_names::KEYWORD, 197, 134, 192),
        color(capture_names::FUNCTION, 220, 220, 170),
        color(capture_names::VARIABLE, 156, 220, 254),
        color(capture_names::CONSTANT, 79, 193, 255),
        color(capture_names::TYPE, 78, 201, 176),
        color(capture_names::OPERATOR, 212, 212, 212),
        color(capture_names::PUNCTUATION_BRACKET, 212, 212, 212),
        color(capture_names::ERROR, 244, 71, 71),
    ])
}

fn light_colors() -> HashMap<String, Color> {
    HashMap::from([
        color(capture_names::COMMENT, 0, 128, 0),
        color(capture_names::STRING, 163, 21, 21),
        color(capture_names::NUMBER, 9, 134, 88),
        color(capture_names::KEYWORD, 0, 0, 255),
        color(capture_names::FUNCTION, 121, 94, 38),
        color(capture_names::VARIABLE, 0, 16, 128),
        color(capture_names::CONSTANT, 38, 127, 153),
        color(capture_names::TYPE, 38, 127, 153),
        color(capture_names::OPERATOR, 0, 0, 0),
        color(capture_names::PUNCTUATION_BRACKET, 0, 0, 0),
        color(capture_names::ERROR, 255, 0, 0),
    ])
}

fn color(capture_name: &str, r: u8, g: u8, b: u8) -> (String, Color) {
    (capture_name.to_string(), Color::new(r, g, b))
}
