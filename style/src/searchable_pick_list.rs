//! Display fields that can be filled with text.
use iced_core::{Background, Color};

use crate::{menu, pick_list, text_input};

/// A set of rules that dictate the style of a searchable_pick_list.
pub trait StyleSheet {
    fn menu(&self) -> menu::Style;

    fn icon_size(&self) -> f32 {
        0.7
    }

    fn text_input_active(&self) -> text_input::Style;

    fn text_input_focused(&self) -> text_input::Style;

    fn text_input_placeholder_color(&self) -> Color;

    fn text_input_value_color(&self) -> Color;

    fn text_input_selection_color(&self) -> Color;

    fn text_input_hovered(&self) -> text_input::Style {
        self.text_input_active()
    }

    fn pick_list_active(&self) -> pick_list::Style;

    fn pick_list_hovered(&self) -> pick_list::Style;
}

struct Default;

impl StyleSheet for Default {
    fn menu(&self) -> menu::Style {
        menu::Style::default()
    }

    fn text_input_active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::WHITE),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn text_input_focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.text_input_active()
        }
    }

    fn text_input_placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn text_input_value_color(&self) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn text_input_selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }

    fn pick_list_active(&self) -> pick_list::Style {
        todo!()
    }

    fn pick_list_hovered(&self) -> pick_list::Style {
        todo!()
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
