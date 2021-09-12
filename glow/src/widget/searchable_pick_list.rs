//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
pub use iced_graphics::searchable_pick_list::StyleSheet;
pub use iced_native::searchable_pick_list::State;

use crate::Renderer;

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type SearchablePickList<'a, T, Message> =
    iced_native::SearchablePickList<'a, T, Message, Renderer>;
