use iced::{
    Element, Length, Rectangle, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{Limits, Node},
        renderer,
        widget::tree::{self, Tree},
    },
    event::Event,
    mouse::{Cursor, Interaction},
};

pub struct SelectableText<'a> {
    content: &'a str,
    size: Option<f32>,
}

#[derive(Default)]
struct State {
    /// Character-offset range of the current selection (anchor, focus).
    selection: Option<(usize, usize)>,
    is_dragging: bool,
}

pub fn selectable_text(content: &str) -> SelectableText<'_> {
    SelectableText {
        content,
        size: None,
    }
}

impl<'a> SelectableText<'a> {
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }
}

impl<'a, Message, R> Widget<Message, Theme, R> for SelectableText<'a>
where
    R: iced::advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &R, _limits: &Limits) -> Node {
        todo!()
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut R,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        todo!()
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &R,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        todo!()
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &R,
    ) -> Interaction {
        todo!()
    }
}

impl<'a, Message, R> From<SelectableText<'a>> for Element<'a, Message, Theme, R>
where
    R: iced::advanced::text::Renderer + 'a,
    Message: 'a,
{
    fn from(widget: SelectableText<'a>) -> Self {
        Element::new(widget)
    }
}
