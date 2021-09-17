//! Display fields that can be filled with text.
//!
//! A [`SearchablePickList`] has some local [`State`].
use std::borrow::Cow;

use crate::event::{self, Event};
use crate::layout;
use crate::mouse::{self, click};
use crate::overlay;
use crate::overlay::menu::{self, Menu};
use crate::text;
use crate::touch;
use crate::widget::text_input_shared::cursor::Cursor;
use crate::widget::text_input_shared::editor::Editor;
use crate::widget::text_input_shared::value::Value;
use crate::{keyboard, pick_list};
use crate::{
    Clipboard, Element, Hasher, Layout, Length, Padding, Point, Rectangle,
    Size, Widget,
};

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_native::{text_input_shared, renderer::Null};
/// #
/// # pub type SearchablePickList<'a, Message> = iced_native::SearchablePickList<'a, Message, Null>;
/// #[derive(Debug, Clone)]
/// enum Message {
///     SearchablePickListChanged(String),
/// }
///
/// let mut state = text_input_shared::State::new();
/// let value = "Some text";
///
/// let input = SearchablePickList::new(
///     &mut state,
///     "This is the placeholder...",
///     value,
///     Message::SearchablePickListChanged,
/// )
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct SearchablePickList<'a, T, Message, Renderer: self::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    state: &'a mut State<T>,
    // Text Input
    placeholder: String,
    value: Value,
    font: Renderer::Font,
    width: Length,
    max_width: u32,
    padding: Padding,
    size: Option<u16>,
    on_change: Box<dyn Fn(String) -> Message>,
    on_submit: Option<Message>,
    select_all_first_click: bool,
    // Pick List
    options: Cow<'a, [T]>,
    selected: Option<T>,
    on_selected: Box<dyn Fn(T) -> Message>,
    // Style
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, T: 'a, Message, Renderer> SearchablePickList<'a, T, Message, Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`SearchablePickList`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`SearchablePickList`] changes
    pub fn new<F>(
        state: &'a mut State<T>,
        placeholder: &str,
        value: &str,
        selected: Option<T>,
        options: impl Into<Cow<'a, [T]>>,
        on_change: F,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        SearchablePickList {
            state,
            // Text Input
            placeholder: String::from(placeholder),
            value: Value::new(value),
            font: Default::default(),
            width: Length::Fill,
            max_width: u32::MAX,
            padding: Padding::ZERO,
            size: None,
            on_change: Box::new(on_change),
            on_submit: None,
            select_all_first_click: false,
            // Pick List
            options: options.into(),
            selected,
            on_selected: Box::new(on_selected),
            // Style
            style: Default::default(),
        }
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::widget::text::Renderer::Font
    /// [`Text`]: crate::widget::Text
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }
    /// Sets the width of the [`SearchablePickList`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`SearchablePickList`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the [`Padding`] of the [`SearchablePickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`SearchablePickList`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`SearchablePickList`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the style of the [`SearchablePickList`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the option to select all of the input text on the first click of the [`SearchablePickList`].
    pub fn select_all_first_click(mut self, select: bool) -> Self {
        self.select_all_first_click = select;
        self
    }

    /// Returns the current [`State`] of the [`SearchablePickList`].
    pub fn state(&self) -> &State<T> {
        self.state
    }
}

impl<'a, T, Message, Renderer> SearchablePickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: self::Renderer,
{
    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
        value: Option<&Value>,
    ) -> Renderer::Output {
        let value = value.unwrap_or(&self.value);
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        self::Renderer::draw(
            renderer,
            bounds,
            text_bounds,
            cursor_position,
            self.state.pick_list.is_open,
            self.selected.as_ref().map(ToString::to_string),
            self.font,
            self.size.unwrap_or(renderer.default_size()),
            &self.placeholder,
            self.padding,
            value,
            self.state.is_focused,
            self.state.cursor,
            &self.style,
        )
    }
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for SearchablePickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: Clone,
    Renderer: self::Renderer + 'a,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let text_size = self.size.unwrap_or(renderer.default_size());

        let limits = limits
            .pad(self.padding)
            .width(self.width)
            .max_width(self.max_width)
            .height(Length::Units(text_size));

        let mut text = layout::Node::new(limits.resolve(Size::ZERO));
        text.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(text.size().pad(self.padding), vec![text])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let is_clicked = layout.bounds().contains(cursor_position);

                let event_status = if is_clicked {
                    if !self.state.pick_list.is_open {
                        let selected = self.selected.as_ref();

                        self.state.pick_list.is_open = true;
                        self.state.pick_list.hovered_option = self
                            .options
                            .iter()
                            .position(|option| Some(option) == selected);

                        self.state.is_focused = true;

                        event::Status::Captured
                    } else {
                        let arrow_down_bounds = Rectangle {
                            x: layout.bounds().x + layout.bounds().width
                                - f32::from(self.padding.horizontal())
                                - 30.0,
                            y: layout.bounds().y,
                            ..layout.bounds()
                        };

                        if arrow_down_bounds.contains(cursor_position) {
                            self.state.pick_list.is_open = false;
                            self.state.is_focused = false;

                            event::Status::Captured
                        } else {
                            // Otherwise the user must have clicked inside the text field
                            self.state.is_focused = true;

                            if self.select_all_first_click && !is_clicked {
                                self.state.first_click = true;
                            }

                            let text_layout = layout.children().next().unwrap();
                            let target =
                                cursor_position.x - text_layout.bounds().x;

                            let click = mouse::Click::new(
                                cursor_position,
                                self.state.last_click,
                            );

                            match click.kind() {
                                click::Kind::Single => {
                                    if target > 0.0 {
                                        let value = self.value.clone();

                                        if self.select_all_first_click
                                            && self.state.first_click
                                        {
                                            self.state
                                                .cursor
                                                .select_all(&value);
                                            self.state.first_click = false;
                                        } else {
                                            let position = renderer
                                                .find_cursor_position(
                                                    text_layout.bounds(),
                                                    self.font,
                                                    self.size,
                                                    &value,
                                                    self.state.is_focused,
                                                    self.state.cursor,
                                                    target,
                                                );

                                            self.state.cursor.move_to(position);

                                            self.state.is_dragging = true;
                                        }
                                    } else {
                                        self.state.cursor.move_to(0);

                                        self.state.is_dragging = true;
                                    }
                                }
                                click::Kind::Double => {
                                    let position = renderer
                                        .find_cursor_position(
                                            text_layout.bounds(),
                                            self.font,
                                            self.size,
                                            &self.value,
                                            self.state.is_focused,
                                            self.state.cursor,
                                            target,
                                        );

                                    self.state.cursor.select_range(
                                        self.value
                                            .previous_start_of_word(position),
                                        self.value.next_end_of_word(position),
                                    );

                                    self.state.is_dragging = false;
                                }
                                click::Kind::Triple => {
                                    self.state.cursor.select_all(&self.value);
                                    self.state.is_dragging = false;
                                }
                            }

                            self.state.last_click = Some(click);

                            event::Status::Captured
                        }
                    }
                } else {
                    self.state.pick_list.is_open = false;
                    self.state.is_focused = false;

                    event::Status::Ignored
                };

                if let Some(last_selection) =
                    self.state.pick_list.last_selection.take()
                {
                    messages.push((self.on_selected)(last_selection));

                    self.state.pick_list.is_open = false;
                    self.state.is_focused = false;

                    return event::Status::Captured;
                } else {
                    return event_status;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.state.is_dragging = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if self.state.is_dragging {
                    let text_layout = layout.children().next().unwrap();
                    let target = position.x - text_layout.bounds().x;

                    if target > 0.0 {
                        let value = self.value.clone();

                        let position = renderer.find_cursor_position(
                            text_layout.bounds(),
                            self.font,
                            self.size,
                            &value,
                            self.state.is_focused,
                            self.state.cursor,
                            target,
                        );

                        self.state.cursor.select_range(
                            self.state.cursor.start(&value),
                            position,
                        );
                    }

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if self.state.is_focused
                    && self.state.is_pasting.is_none()
                    && !self.state.keyboard_modifiers.command()
                    && !c.is_control() =>
            {
                let mut editor =
                    Editor::new(&mut self.value, &mut self.state.cursor);

                editor.insert(c);

                let message = (self.on_change)(editor.contents());
                messages.push(message);

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code, ..
            }) if self.state.is_focused => {
                let modifiers = self.state.keyboard_modifiers;

                match key_code {
                    keyboard::KeyCode::Enter => {
                        if let Some(on_submit) = self.on_submit.clone() {
                            messages.push(on_submit);
                        }
                    }
                    keyboard::KeyCode::Backspace => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && self
                                .state
                                .cursor
                                .selection(&self.value)
                                .is_none()
                        {
                            self.state.cursor.select_left_by_words(&self.value);
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.backspace();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::Delete => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && self
                                .state
                                .cursor
                                .selection(&self.value)
                                .is_none()
                        {
                            self.state
                                .cursor
                                .select_right_by_words(&self.value);
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.delete();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::Left => {
                        if platform::is_jump_modifier_pressed(modifiers) {
                            if modifiers.shift() {
                                self.state
                                    .cursor
                                    .select_left_by_words(&self.value);
                            } else {
                                self.state
                                    .cursor
                                    .move_left_by_words(&self.value);
                            }
                        } else if modifiers.shift() {
                            self.state.cursor.select_left(&self.value)
                        } else {
                            self.state.cursor.move_left(&self.value);
                        }
                    }
                    keyboard::KeyCode::Right => {
                        if platform::is_jump_modifier_pressed(modifiers) {
                            if modifiers.shift() {
                                self.state
                                    .cursor
                                    .select_right_by_words(&self.value);
                            } else {
                                self.state
                                    .cursor
                                    .move_right_by_words(&self.value);
                            }
                        } else if modifiers.shift() {
                            self.state.cursor.select_right(&self.value)
                        } else {
                            self.state.cursor.move_right(&self.value);
                        }
                    }
                    keyboard::KeyCode::Home => {
                        if modifiers.shift() {
                            self.state.cursor.select_range(
                                self.state.cursor.start(&self.value),
                                0,
                            );
                        } else {
                            self.state.cursor.move_to(0);
                        }
                    }
                    keyboard::KeyCode::End => {
                        if modifiers.shift() {
                            self.state.cursor.select_range(
                                self.state.cursor.start(&self.value),
                                self.value.len(),
                            );
                        } else {
                            self.state.cursor.move_to(self.value.len());
                        }
                    }
                    keyboard::KeyCode::C
                        if self.state.keyboard_modifiers.command() =>
                    {
                        match self.state.cursor.selection(&self.value) {
                            Some((start, end)) => {
                                clipboard.write(
                                    self.value.select(start, end).to_string(),
                                );
                            }
                            None => {}
                        }
                    }
                    keyboard::KeyCode::X
                        if self.state.keyboard_modifiers.command() =>
                    {
                        match self.state.cursor.selection(&self.value) {
                            Some((start, end)) => {
                                clipboard.write(
                                    self.value.select(start, end).to_string(),
                                );
                            }
                            None => {}
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.delete();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::V => {
                        if self.state.keyboard_modifiers.command() {
                            let content = match self.state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read()
                                        .unwrap_or(String::new())
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let mut editor = Editor::new(
                                &mut self.value,
                                &mut self.state.cursor,
                            );

                            editor.paste(content.clone());

                            let message = (self.on_change)(editor.contents());
                            messages.push(message);

                            self.state.is_pasting = Some(content);
                        } else {
                            self.state.is_pasting = None;
                        }
                    }
                    keyboard::KeyCode::A
                        if self.state.keyboard_modifiers.command() =>
                    {
                        self.state.cursor.select_all(&self.value);
                    }
                    keyboard::KeyCode::Escape => {
                        self.state.is_focused = false;
                        self.state.is_dragging = false;
                        self.state.is_pasting = None;

                        self.state.keyboard_modifiers =
                            keyboard::Modifiers::default();
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code, ..
            }) if self.state.is_focused => {
                match key_code {
                    keyboard::KeyCode::V => {
                        self.state.is_pasting = None;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers))
                if self.state.is_focused =>
            {
                self.state.keyboard_modifiers = modifiers;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        self.draw(renderer, layout, cursor_position, None)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::{any::TypeId, hash::Hash};
        struct Marker;
        TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.max_width.hash(state);
        self.padding.hash(state);
        self.size.hash(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        if self.state.pick_list.is_open {
            let bounds = layout.bounds();

            let mut menu = Menu::new(
                &mut self.state.pick_list.menu,
                &self.options,
                &mut self.state.pick_list.hovered_option,
                &mut self.state.pick_list.last_selection,
            )
            .width(bounds.width.round() as u16)
            .padding(self.padding)
            .font(self.font)
            .style(Renderer::menu_style(&self.style));

            if let Some(size) = self.size {
                menu = menu.text_size(size);
            }

            Some(menu.overlay(layout.position(), bounds.height))
        } else {
            None
        }
    }
}

/// The renderer of a [`SearchablePickList`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`SearchablePickList`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: text::Renderer + menu::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the width of the value of the [`SearchablePickList`].
    fn measure_value(&self, value: &str, size: u16, font: Self::Font) -> f32;

    /// Returns the current horizontal offset of the value of the
    /// [`SearchablePickList`].
    ///
    /// This is the amount of horizontal scrolling applied when the [`Value`]
    /// does not fit the [`SearchablePickList`].
    fn offset(
        &self,
        text_bounds: Rectangle,
        font: Self::Font,
        size: u16,
        value: &Value,
        is_focused: bool,
        cursor: Cursor,
    ) -> f32;

    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as menu::Renderer>::Style;

    /// Draws a [`SearchablePickList`].
    ///
    /// It receives:
    /// - the bounds of the [`SearchablePickList`]
    /// - the bounds of the text (i.e. the current value)
    /// - the cursor position
    /// - the placeholder to show when the value is empty
    /// - the current [`Value`]
    /// - the current [`State`]
    fn draw(
        &mut self,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
        pick_list_is_open: bool,
        selected: Option<String>,
        font: Self::Font,
        size: u16,
        placeholder: &str,
        padding: Padding,
        value: &Value,
        is_focused: bool,
        cursor: Cursor,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;

    /// Computes the position of the text cursor at the given X coordinate of
    /// a [`SearchablePickList`].
    fn find_cursor_position(
        &self,
        text_bounds: Rectangle,
        font: Self::Font,
        size: Option<u16>,
        value: &Value,
        is_focused: bool,
        cursor: Cursor,
        x: f32,
    ) -> usize {
        let size = size.unwrap_or(self.default_size());

        let offset =
            self.offset(text_bounds, font, size, &value, is_focused, cursor);

        find_cursor_position(
            self,
            &value,
            font,
            size,
            x + offset,
            0,
            value.len(),
        )
    }
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for SearchablePickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: self::Renderer + 'a,
    Message: 'a + Clone,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}

/// The state of a [`SearchablePickList`].
#[derive(Debug, Default, Clone)]
pub struct State<T> {
    pick_list: pick_list::State<T>,
    is_focused: bool,
    is_dragging: bool,
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    first_click: bool,
    // TODO: Add stateful horizontal scrolling offset
}

impl<T: Default> State<T> {
    /// Creates a new [`State`], representing an unfocused [`SearchablePickList`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`SearchablePickList`].
    pub fn focused() -> Self {
        Self {
            pick_list: pick_list::State::default(),
            is_focused: true,
            is_dragging: false,
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            first_click: false,
        }
    }

    /// Returns whether the [`SearchablePickList`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Returns the [`Cursor`] of the [`SearchablePickList`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`SearchablePickList`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`SearchablePickList`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }

    /// Moves the [`Cursor`] of the [`SearchablePickList`] to the front of the input text.
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`SearchablePickList`] to the end of the input text.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`SearchablePickList`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`SearchablePickList`].
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }
}

// TODO: Reduce allocations
fn find_cursor_position<Renderer: self::Renderer>(
    renderer: &Renderer,
    value: &Value,
    font: Renderer::Font,
    size: u16,
    target: f32,
    start: usize,
    end: usize,
) -> usize {
    if start >= end {
        if start == 0 {
            return 0;
        }

        let prev = value.until(start - 1);
        let next = value.until(start);

        let prev_width = renderer.measure_value(&prev.to_string(), size, font);
        let next_width = renderer.measure_value(&next.to_string(), size, font);

        if next_width - target > target - prev_width {
            return start - 1;
        } else {
            return start;
        }
    }

    let index = (end - start) / 2;
    let subvalue = value.until(start + index);

    let width = renderer.measure_value(&subvalue.to_string(), size, font);

    if width > target {
        find_cursor_position(
            renderer,
            value,
            font,
            size,
            target,
            start,
            start + index,
        )
    } else {
        find_cursor_position(
            renderer,
            value,
            font,
            size,
            target,
            start + index + 1,
            end,
        )
    }
}

mod platform {
    use crate::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}
