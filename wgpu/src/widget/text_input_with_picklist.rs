//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
pub use iced_graphics::text_input_with_picklist::{Style, StyleSheet};
pub use iced_native::text_input_with_picklist::State;

use crate::Renderer;

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type TextInputWithPickList<'a, T, Message> =
    iced_native::TextInputWithPickList<'a, T, Message, Renderer>;
