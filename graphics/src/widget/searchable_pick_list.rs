//! Display fields that can be filled with text.
//!
//! A [`SearchablePickList`] has some local [`State`].
use std::f32;

pub use iced_native::searchable_pick_list::State;
use iced_native::searchable_pick_list::{self};
use iced_native::text_input_shared::cursor;
use iced_native::{mouse, text_input_shared, Padding};
use iced_native::{
    Background, Color, Font, HorizontalAlignment, Point, Rectangle, Size,
    Vector, VerticalAlignment,
};
use iced_style::menu;
pub use iced_style::searchable_pick_list::StyleSheet;

use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with pick list with an `iced_wgpu::Renderer`.
pub type SearchablePickList<'a, T, Message, Backend> =
    iced_native::SearchablePickList<'a, T, Message, Renderer<Backend>>;

impl<B> searchable_pick_list::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    fn measure_value(&self, value: &str, size: u16, font: Font) -> f32 {
        let backend = self.backend();

        let (width, _) =
            backend.measure(value, f32::from(size), font, Size::INFINITY);

        width
    }

    fn menu_style(style: &Box<dyn StyleSheet>) -> menu::Style {
        style.menu()
    }

    fn offset(
        &self,
        text_bounds: Rectangle,
        font: Font,
        size: u16,
        value: &text_input_shared::value::Value,
        is_focused: bool,
        cursor: text_input_shared::cursor::Cursor,
    ) -> f32 {
        if is_focused {
            let focus_position = match cursor.state(value) {
                cursor::State::Index(i) => i,
                cursor::State::Selection { end, .. } => end,
            };

            let (_, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                focus_position,
                font,
            );

            offset
        } else {
            0.0
        }
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        mut text_bounds: Rectangle,
        cursor_position: Point,
        pick_list_is_open: bool,
        selected: Option<String>,
        font: Font,
        size: u16,
        placeholder: &str,
        padding: Padding,
        value: &text_input_shared::value::Value,
        is_focused: bool,
        cursor: text_input_shared::cursor::Cursor,
        style_sheet: &Box<dyn StyleSheet>,
    ) -> Self::Output {
        if pick_list_is_open {
            text_bounds.width -= 30.0;

            let is_mouse_over_text = bounds.contains(cursor_position);

            let style = if is_focused {
                style_sheet.text_input_focused()
            } else if is_mouse_over_text {
                style_sheet.text_input_hovered()
            } else {
                style_sheet.text_input_active()
            };

            let arrow_down_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(padding.horizontal())
                    - 30.0,
                y: bounds.y,
                ..bounds
            };

            let arrow_down = Primitive::Text {
                content: B::ARROW_DOWN_ICON.to_string(),
                font: B::ICON_FONT,
                size: bounds.height * style_sheet.icon_size(),
                bounds: Rectangle {
                    x: bounds.x + bounds.width
                        - f32::from(padding.horizontal()),
                    y: bounds.center_y(),
                    ..bounds
                },
                color: style_sheet.text_input_value_color(),
                horizontal_alignment: HorizontalAlignment::Right,
                vertical_alignment: VerticalAlignment::Center,
            };

            let is_mouse_over_arrow_down =
                arrow_down_bounds.contains(cursor_position);

            let input = Primitive::Quad {
                bounds,
                background: style.background,
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            };

            let text = value.to_string();

            let text_value = Primitive::Text {
                content: if text.is_empty() {
                    placeholder.to_string()
                } else {
                    text.clone()
                },
                color: if text.is_empty() {
                    style_sheet.text_input_placeholder_color()
                } else {
                    style_sheet.text_input_value_color()
                },
                font,
                bounds: Rectangle {
                    y: text_bounds.center_y(),
                    width: f32::INFINITY,
                    ..text_bounds
                },
                size: f32::from(size),
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            };

            let (contents_primitive, offset) = if is_focused {
                let (cursor_primitive, offset) = match cursor.state(value) {
                    cursor::State::Index(position) => {
                        let (text_value_width, offset) =
                            measure_cursor_and_scroll_offset(
                                self,
                                text_bounds,
                                value,
                                size,
                                position,
                                font,
                            );

                        (
                            Primitive::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + text_value_width,
                                    y: text_bounds.y,
                                    width: 1.0,
                                    height: text_bounds.height,
                                },
                                background: Background::Color(
                                    style_sheet.text_input_value_color(),
                                ),
                                border_radius: 0.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            offset,
                        )
                    }
                    cursor::State::Selection { start, end } => {
                        let left = start.min(end);
                        let right = end.max(start);

                        let (left_position, left_offset) =
                            measure_cursor_and_scroll_offset(
                                self,
                                text_bounds,
                                value,
                                size,
                                left,
                                font,
                            );

                        let (right_position, right_offset) =
                            measure_cursor_and_scroll_offset(
                                self,
                                text_bounds,
                                value,
                                size,
                                right,
                                font,
                            );

                        let width = right_position - left_position;

                        (
                            Primitive::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                background: Background::Color(
                                    style_sheet.text_input_selection_color(),
                                ),
                                border_radius: 0.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            if end == right {
                                right_offset
                            } else {
                                left_offset
                            },
                        )
                    }
                };

                (
                    Primitive::Group {
                        primitives: vec![cursor_primitive, text_value],
                    },
                    Vector::new(offset as u32, 0),
                )
            } else {
                (text_value, Vector::new(0, 0))
            };

            let text_width = self.measure_value(
                if text.is_empty() { placeholder } else { &text },
                size,
                font,
            );

            let contents = if text_width > text_bounds.width {
                Primitive::Clip {
                    bounds: text_bounds,
                    offset,
                    content: Box::new(contents_primitive),
                }
            } else {
                contents_primitive
            };

            (
                Primitive::Group {
                    primitives: vec![input, contents, arrow_down],
                },
                if is_mouse_over_arrow_down {
                    mouse::Interaction::Pointer
                } else if is_mouse_over_text {
                    mouse::Interaction::Text
                } else {
                    mouse::Interaction::default()
                },
            )
        } else {
            let is_mouse_over = bounds.contains(cursor_position);
            let is_selected = selected.is_some();

            let style = if is_mouse_over {
                style_sheet.pick_list_hovered()
            } else {
                style_sheet.pick_list_active()
            };

            let background = Primitive::Quad {
                bounds,
                background: style.background,
                border_color: style.border_color,
                border_width: style.border_width,
                border_radius: style.border_radius,
            };

            let arrow_down = Primitive::Text {
                content: B::ARROW_DOWN_ICON.to_string(),
                font: B::ICON_FONT,
                size: bounds.height * style.icon_size,
                bounds: Rectangle {
                    x: bounds.x + bounds.width
                        - f32::from(padding.horizontal()),
                    y: bounds.center_y(),
                    ..bounds
                },
                color: style_sheet.text_input_value_color(),
                horizontal_alignment: HorizontalAlignment::Right,
                vertical_alignment: VerticalAlignment::Center,
            };

            (
                Primitive::Group {
                    primitives: if let Some(label) =
                        selected.or_else(|| Some(placeholder.to_string()))
                    {
                        let label = Primitive::Text {
                            content: label,
                            size: f32::from(size),
                            font,
                            color: is_selected
                                .then(|| style_sheet.text_input_value_color())
                                .unwrap_or(
                                    style_sheet.text_input_placeholder_color(),
                                ),
                            bounds: Rectangle {
                                x: bounds.x + f32::from(padding.left),
                                y: bounds.center_y(),
                                ..bounds
                            },
                            horizontal_alignment: HorizontalAlignment::Left,
                            vertical_alignment: VerticalAlignment::Center,
                        };

                        vec![background, label, arrow_down]
                    } else {
                        vec![background, arrow_down]
                    },
                },
                if is_mouse_over {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::default()
                },
            )
        }
    }
}

fn measure_cursor_and_scroll_offset<B>(
    renderer: &Renderer<B>,
    text_bounds: Rectangle,
    value: &text_input_shared::value::Value,
    size: u16,
    cursor_index: usize,
    font: Font,
) -> (f32, f32)
where
    B: Backend + backend::Text,
{
    use iced_native::searchable_pick_list::Renderer;

    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_value(&text_before_cursor, size, font);
    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}
