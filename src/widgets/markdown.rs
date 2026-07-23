use iced::font::{self, Font};
use iced::theme::palette;
use iced::widget::{span, text};
use iced::{Color, Element, Length, Padding, Pixels, Theme};
use iced::{alignment, border, color, padding};
use iced_graphics::core;

use std::cell::{Cell, RefCell};
use std::mem;
use std::sync::Arc;

use iced::advanced::text::Paragraph as ParagraphTrait;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget,
    clipboard::Kind as ClipboardKind,
    layout::{Limits, Node},
    mouse::{Click, click},
    renderer,
    widget::tree::{self, Tree},
};
use iced::event::Event;
use iced::keyboard;
use iced::mouse::{self, Cursor, Interaction};
use iced_graphics::text::Paragraph as ConcreteP;
use regex::Regex;

use crate::widgets::text_utils;

pub use iced::advanced::text::Highlight;
pub use pulldown_cmark::HeadingLevel;

/// A Markdown item.
#[derive(Debug, Clone)]
pub enum Item<Message> {
    /// A heading.
    Heading(pulldown_cmark::HeadingLevel, Text<Message>),
    /// A paragraph.
    Paragraph(Text<Message>),
    /// A code block.
    CodeBlock(String),
    /// A list.
    List {
        /// The first number of the list, if it is ordered.
        start: Option<u64>,
        /// The items of the list.
        bullets: Vec<Bullet<Message>>,
    },
    /// A horizontal separator.
    Rule,
}

/// A bunch of parsed Markdown text.
#[derive(Debug, Clone)]
pub struct Text<Message> {
    spans: Vec<Span>,
    last_style: Cell<Option<Style>>,
    last_styled_spans: RefCell<Arc<[text::Span<'static, Message>]>>,
}

impl<Message> Text<Message> {
    fn new(spans: Vec<Span>) -> Self {
        Self {
            spans,
            last_style: Cell::default(),
            last_styled_spans: RefCell::default(),
        }
    }

    /// Returns the [`rich_text()`] spans ready to be used for the given style.
    ///
    /// This method performs caching for you. It will only reallocate if the [`Style`]
    /// provided changes.
    pub fn spans(&self, style: Style) -> Arc<[text::Span<'static, Message>]> {
        if Some(style) != self.last_style.get() {
            *self.last_styled_spans.borrow_mut() = self
                .spans
                .iter()
                .map(|span| span.view::<Message>(&style))
                .collect();

            self.last_style.set(Some(style));
        }

        self.last_styled_spans.borrow().clone()
    }
}

#[derive(Debug, Clone)]
enum Span {
    Standard {
        text: String,
        strikethrough: bool,
        strong: bool,
        emphasis: bool,
        code: bool,
    },
}

impl Span {
    fn view<Message>(&self, style: &Style) -> text::Span<'static, Message> {
        match self {
            Span::Standard {
                text,
                strikethrough,
                strong,
                emphasis,
                code,
            } => {
                let span = span(text.clone()).strikethrough(*strikethrough);

                if *code {
                    span.font(style.inline_code_font)
                        .color(style.inline_code_color)
                        .background(style.inline_code_highlight.background)
                        .border(style.inline_code_highlight.border)
                        .padding(style.inline_code_padding)
                } else if *strong || *emphasis {
                    span.font(Font {
                        weight: if *strong {
                            font::Weight::Bold
                        } else {
                            font::Weight::Normal
                        },
                        style: if *emphasis {
                            font::Style::Italic
                        } else {
                            font::Style::Normal
                        },
                        ..style.font
                    })
                } else {
                    span.font(style.font)
                }
            }
        }
    }
}

/// The item of a list.
#[derive(Debug, Clone)]
pub struct Bullet<Message> {
    pub items: Vec<Item<Message>>,
}

impl<Message> Bullet<Message> {
    fn items(&self) -> &[Item<Message>] {
        &self.items
    }

    fn push(&mut self, item: Item<Message>) {
        self.items.push(item);
    }
}

pub fn parse<Message>(markdown: &str) -> impl Iterator<Item = Item<Message>> + '_
where
    Message: 'static,
{
    parse_with(markdown)
}

fn parse_with<Message>(markdown: &str) -> impl Iterator<Item = Item<Message>> + '_
where
    Message: 'static,
{
    enum Scope<Message> {
        List(List<Message>),
    }

    struct List<Message> {
        start: Option<u64>,
        bullets: Vec<Bullet<Message>>,
    }

    let mut spans = Vec::new();
    let mut code = String::new();
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
    let mut metadata = false;
    let mut code_block = false;
    let mut stack = Vec::new();

    let parser = pulldown_cmark::Parser::new_ext(
        markdown,
        pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH,
    );

    let produce = move |stack: &mut Vec<Scope<Message>>, item: Item<Message>| {
        if let Some(scope) = stack.last_mut() {
            match scope {
                Scope::List(list) => {
                    list.bullets.last_mut()?.push(item);
                }
            }
            None
        } else {
            Some(item)
        }
    };

    let parser = parser.into_offset_iter();

    // We want to keep the `spans` capacity
    #[allow(clippy::drain_collect)]
    parser.filter_map(move |(event, _source)| match event {
        pulldown_cmark::Event::Start(tag) => match tag {
            pulldown_cmark::Tag::Strong if !metadata => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::Emphasis if !metadata => {
                emphasis = true;
                None
            }
            pulldown_cmark::Tag::Strikethrough if !metadata => {
                strikethrough = true;
                None
            }
            pulldown_cmark::Tag::List(first_item) if !metadata => {
                let prev = if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                    )
                };

                stack.push(Scope::List(List {
                    start: first_item,
                    bullets: Vec::new(),
                }));

                prev
            }
            pulldown_cmark::Tag::Item => {
                if let Some(Scope::List(list)) = stack.last_mut() {
                    list.bullets.push(Bullet { items: Vec::new() });
                }

                None
            }
            pulldown_cmark::Tag::CodeBlock(_) if !metadata => {
                code_block = true;

                if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                    )
                }
            }
            pulldown_cmark::Tag::MetadataBlock(_) => {
                metadata = true;
                None
            }
            _ => None,
        },
        pulldown_cmark::Event::End(tag) => match tag {
            pulldown_cmark::TagEnd::Heading(level) if !metadata => produce(
                &mut stack,
                Item::Heading(level, Text::new(spans.drain(..).collect())),
            ),
            pulldown_cmark::TagEnd::Strong if !metadata => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::Emphasis if !metadata => {
                emphasis = false;
                None
            }
            pulldown_cmark::TagEnd::Strikethrough if !metadata => {
                strikethrough = false;
                None
            }
            pulldown_cmark::TagEnd::Paragraph if !metadata => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                    )
                }
            }
            pulldown_cmark::TagEnd::Item if !metadata => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                    )
                }
            }
            pulldown_cmark::TagEnd::List(_) if !metadata => {
                let Scope::List(list) = stack.pop()?;

                produce(
                    &mut stack,
                    Item::List {
                        start: list.start,
                        bullets: list.bullets,
                    },
                )
            }
            pulldown_cmark::TagEnd::CodeBlock if !metadata => {
                code_block = false;
                produce(&mut stack, Item::CodeBlock(mem::take(&mut code)))
            }
            pulldown_cmark::TagEnd::MetadataBlock(_) => {
                metadata = false;
                None
            }
            _ => None,
        },
        pulldown_cmark::Event::Text(text) if !metadata => {
            if code_block {
                code.push_str(&text);
                return None;
            }

            let span = Span::Standard {
                text: text.into_string(),
                strong,
                emphasis,
                strikethrough,
                code: false,
            };

            spans.push(span);

            None
        }
        pulldown_cmark::Event::Code(code) if !metadata => {
            let span = Span::Standard {
                text: code.into_string(),
                strong,
                emphasis,
                strikethrough,
                code: true,
            };

            spans.push(span);
            None
        }
        pulldown_cmark::Event::SoftBreak if !metadata => {
            spans.push(Span::Standard {
                text: String::from("\n"),
                strikethrough,
                strong,
                emphasis,
                code: false,
            });
            None
        }
        pulldown_cmark::Event::HardBreak if !metadata => {
            spans.push(Span::Standard {
                text: String::from("\n"),
                strikethrough,
                strong,
                emphasis,
                code: false,
            });
            None
        }
        pulldown_cmark::Event::Rule => produce(&mut stack, Item::Rule),
        _ => None,
    })
}

/// Configuration controlling Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// The base text size.
    pub text_size: Pixels,
    /// The text size of level 1 heading.
    pub h1_size: Pixels,
    /// The text size of level 2 heading.
    pub h2_size: Pixels,
    /// The text size of level 3 heading.
    pub h3_size: Pixels,
    /// The text size of level 4 heading.
    pub h4_size: Pixels,
    /// The text size of level 5 heading.
    pub h5_size: Pixels,
    /// The text size of level 6 heading.
    pub h6_size: Pixels,
    /// The styling of the Markdown.
    pub style: Style,
}

impl Settings {
    /// Creates new [`Settings`] with default text size and the given [`Style`].
    pub fn with_style(style: impl Into<Style>) -> Self {
        Self::with_text_size(16, style)
    }

    /// Creates new [`Settings`] with the given base text size in [`Pixels`].
    ///
    /// Heading levels will be adjusted automatically. Specifically,
    /// the first level will be twice the base size, and then every level
    /// after that will be 25% smaller.
    pub fn with_text_size(text_size: impl Into<Pixels>, style: impl Into<Style>) -> Self {
        let text_size = text_size.into();

        Self {
            text_size,
            h1_size: text_size * 2.0,
            h2_size: text_size * 1.75,
            h3_size: text_size * 1.5,
            h4_size: text_size * 1.25,
            h5_size: text_size,
            h6_size: text_size,
            style: style.into(),
        }
    }
}

impl From<&Theme> for Settings {
    fn from(theme: &Theme) -> Self {
        Self::with_style(Style::from(theme))
    }
}

impl From<Theme> for Settings {
    fn from(theme: Theme) -> Self {
        Self::with_style(Style::from(theme))
    }
}

/// The text styling of some Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Font`] to be applied to basic text.
    pub font: Font,
    /// The [`Highlight`] to be applied to the background of inline code.
    pub inline_code_highlight: Highlight,
    /// The [`Padding`] to be applied to the background of inline code.
    pub inline_code_padding: Padding,
    /// The [`Color`] to be applied to inline code.
    pub inline_code_color: Color,
    /// The [`Font`] to be applied to inline code.
    pub inline_code_font: Font,
    /// The [`Font`] to be applied to code blocks.
    pub code_block_font: Font,
}

impl Style {
    /// Creates a new [`Style`] from the given [`palette::Palette`].
    pub fn from_palette(_palette: palette::Palette) -> Self {
        Self {
            font: Font::default(),
            inline_code_padding: padding::left(1).right(1),
            inline_code_highlight: Highlight {
                background: color!(0x111111).into(),
                border: border::rounded(4),
            },
            inline_code_color: Color::WHITE,
            inline_code_font: Font::MONOSPACE,
            code_block_font: Font::MONOSPACE,
        }
    }
}

impl From<palette::Palette> for Style {
    fn from(palette: palette::Palette) -> Self {
        Self::from_palette(palette)
    }
}

impl From<&Theme> for Style {
    fn from(theme: &Theme) -> Self {
        Self::from_palette(theme.palette())
    }
}

impl From<Theme> for Style {
    fn from(theme: Theme) -> Self {
        Self::from_palette(theme.palette())
    }
}

// ─────────────────────────────── Selectable Markdown ───────────────────────────────

fn fill_run_quad<R>(
    renderer: &mut R,
    bounds: iced::Rectangle,
    run: &cosmic_text::LayoutRun<'_>,
    from: usize,
    to: usize,
    color: Color,
) where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font>,
{
    let (x, width) = text_utils::highlight_run(run, from, to);
    if width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds: iced::Rectangle {
                    x: bounds.x + x,
                    y: bounds.y + run.line_top,
                    width,
                    height: run.line_height,
                },
                ..renderer::Quad::default()
            },
            color,
        );
    }
}

struct SelectableRichTextState<Message> {
    paragraph: ConcreteP,
    prev_spans: Vec<text::Span<'static, Message>>,
    selection: Option<((usize, usize), (usize, usize))>,
    last_click: Option<Click>,
    is_dragging: bool,
}

/// A markdown text element that supports mouse selection and pattern highlighting.
pub struct SelectableRichText<Message> {
    spans: Arc<[text::Span<'static, Message>]>,
    size: Pixels,
    font: Font,
    highlight_patterns: Vec<(String, Arc<dyn Fn(&Theme) -> Color>)>,
    computed_highlights: Vec<(usize, usize, usize, usize)>,
    rule_lines: Vec<usize>,
    heading_lines: Vec<usize>,
}

impl<Message> SelectableRichText<Message> {
    fn new(spans: Arc<[text::Span<'static, Message>]>, size: Pixels, font: Font) -> Self {
        Self {
            spans,
            size,
            font,
            highlight_patterns: Vec::new(),
            computed_highlights: Vec::new(),
            rule_lines: Vec::new(),
            heading_lines: Vec::new(),
        }
    }

    fn with_rule_lines(mut self, rule_lines: Vec<usize>) -> Self {
        self.rule_lines = rule_lines;
        self
    }

    fn with_heading_lines(mut self, heading_lines: Vec<usize>) -> Self {
        self.heading_lines = heading_lines;
        self
    }

    fn highlight_str_arc(mut self, pattern: String, color: Arc<dyn Fn(&Theme) -> Color>) -> Self {
        if pattern.is_empty() {
            return self;
        }

        let Ok(re) = Regex::new(&format!("(?i){}", pattern)) else {
            return self;
        };

        let content: String = self.spans.iter().map(|s| s.text.as_ref()).collect();
        let pattern_idx = self.highlight_patterns.len();
        for (line_idx, line) in content.split('\n').enumerate() {
            if self.heading_lines.contains(&line_idx) {
                continue;
            }

            for mat in re.find_iter(line) {
                self.computed_highlights
                    .push((line_idx, mat.start(), mat.end(), pattern_idx));
            }
        }
        self.highlight_patterns.push((pattern, color));
        self
    }
}

impl<Message, R> Widget<Message, Theme, R> for SelectableRichText<Message>
where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font>,
    Message: 'static + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SelectableRichTextState<Message>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SelectableRichTextState::<Message> {
            paragraph: ConcreteP::default(),
            prev_spans: Vec::new(),
            selection: None,
            last_click: None,
            is_dragging: false,
        })
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &R, limits: &Limits) -> Node {
        let state = tree
            .state
            .downcast_mut::<SelectableRichTextState<Message>>();
        let bounds = limits.max();

        let mk_text = || core::Text {
            content: self.spans.as_ref(),
            bounds,
            size: self.size,
            line_height: text::LineHeight::default(),
            font: self.font,
            align_x: core::text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::default(),
        };

        if state.prev_spans.as_slice() != self.spans.as_ref() {
            state.paragraph = ConcreteP::with_spans(mk_text());
            state.prev_spans = self.spans.iter().cloned().collect();
            state.selection = None;
        } else {
            match state.paragraph.compare(core::Text {
                content: (),
                bounds,
                size: self.size,
                line_height: text::LineHeight::default(),
                font: self.font,
                align_x: core::text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            }) {
                core::text::Difference::None => {}
                core::text::Difference::Bounds => {
                    state.paragraph.resize(bounds);
                }
                core::text::Difference::Shape => {
                    state.paragraph = ConcreteP::with_spans(mk_text());
                }
            }
        }

        let min_bounds = state.paragraph.min_bounds();
        Node::new(limits.resolve(Length::Fill, Length::Shrink, min_bounds))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<SelectableRichTextState<Message>>();
        let bounds = layout.bounds();
        let buffer = state.paragraph.buffer();

        // Draw pattern highlights and selection using layout_runs() so each visual line
        // gets its actual pixel y position and height, which vary when spans have
        // different font sizes (e.g. headings vs body text).
        for run in buffer.layout_runs() {
            let run_top = bounds.y + run.line_top;
            let run_bottom = run_top + run.line_height;
            if run_bottom < viewport.y || run_top > viewport.y + viewport.height {
                continue;
            }

            // Pattern highlights
            for &(line_idx, from, to, pattern_idx) in &self.computed_highlights {
                if run.line_i == line_idx {
                    let color = self.highlight_patterns[pattern_idx].1(theme);
                    fill_run_quad(renderer, bounds, &run, from, to, color);
                }
            }

            // Selection highlight
            if let Some(((anchor_line, anchor_idx), (focus_line, focus_idx))) = state.selection {
                let ((start_line, start_idx), (end_line, end_idx)) =
                    text_utils::normalize_selection(anchor_line, anchor_idx, focus_line, focus_idx);

                if (start_line, start_idx) < (end_line, end_idx)
                    && run.line_i >= start_line
                    && run.line_i <= end_line
                {
                    let from = if run.line_i == start_line {
                        start_idx
                    } else {
                        0
                    };
                    let to = if run.line_i == end_line {
                        end_idx
                    } else {
                        buffer.lines[run.line_i].text().len()
                    };
                    let color = theme.extended_palette().primary.weak.color;
                    fill_run_quad(renderer, bounds, &run, from, to, color);
                }
            }

            // Heading rules — drawn as a full-width 1px quad centered in the empty rule line
            if self.rule_lines.contains(&run.line_i) {
                let rule_color = theme.extended_palette().background.strong.color;
                let mid_y = bounds.y + run.line_top + run.line_height / 2.0;
                renderer.fill_quad(
                    iced::advanced::renderer::Quad {
                        bounds: iced::Rectangle {
                            x: bounds.x,
                            y: mid_y,
                            width: bounds.width,
                            height: 2.0,
                        },
                        ..iced::advanced::renderer::Quad::default()
                    },
                    rule_color,
                );
            }
        }

        let anchor = bounds.anchor(
            state.paragraph.min_bounds(),
            state.paragraph.align_x(),
            state.paragraph.align_y(),
        );
        renderer.fill_paragraph(&state.paragraph, anchor, style.text_color, *viewport);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree
            .state
            .downcast_mut::<SelectableRichTextState<Message>>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(mouse_pos) = cursor.position_in(layout.bounds()) {
                    let click = Click::new(mouse_pos, mouse::Button::Left, state.last_click);

                    if let Some(sel) =
                        text_utils::selection_from_click(state.paragraph.buffer(), click, mouse_pos)
                    {
                        state.selection = Some(sel);
                        if click.kind() == click::Kind::Single {
                            state.is_dragging = true;
                        }
                    } else {
                        state.selection = None;
                    }

                    state.last_click = Some(click);
                } else {
                    state.selection = None;
                }
                shell.request_redraw();
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging
                    && state.last_click.map(|c| c.kind()) == Some(click::Kind::Single)
                {
                    if let Some(mouse_pos) = cursor.position_in(layout.bounds()) {
                        if let Some(((anchor_line, anchor_idx), _)) = state.selection {
                            if let Some(new_sel) = text_utils::selection_from_drag(
                                state.paragraph.buffer(),
                                (anchor_line, anchor_idx),
                                mouse_pos,
                            ) {
                                state.selection = Some(new_sel);
                                shell.request_redraw();
                            }
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if c.as_str() == "c" && modifiers.command() => {
                if let Some(((anchor_line, anchor_idx), (focus_line, focus_idx))) = state.selection
                {
                    if let Some(text) = text_utils::extract_selection_text(
                        state.paragraph.buffer(),
                        anchor_line,
                        anchor_idx,
                        focus_line,
                        focus_idx,
                    ) {
                        clipboard.write(ClipboardKind::Standard, text);
                    }
                }
            }

            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &R,
    ) -> Interaction {
        if cursor.is_over(layout.bounds()) {
            Interaction::Text
        } else {
            Interaction::default()
        }
    }
}

impl<'a, Message, R> From<SelectableRichText<Message>> for Element<'a, Message, Theme, R>
where
    R: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = iced::Font> + 'a,
    Message: 'static + Clone,
{
    fn from(w: SelectableRichText<Message>) -> Self {
        Element::new(w)
    }
}

/// A markdown view builder that supports text selection and pattern highlighting.
///
/// Created via [`view_selectable`]. Call [`highlight_str`](SelectableMarkdown::highlight_str)
/// to highlight a search pattern, then convert to [`Element`] via `Into`.
pub struct SelectableMarkdown<Message, Renderer> {
    items: Vec<Item<Message>>,
    settings: Settings,
    highlights: Vec<(String, Arc<dyn Fn(&Theme) -> Color>)>,
    _message: std::marker::PhantomData<Message>,
    _renderer: std::marker::PhantomData<Renderer>,
}

/// Display markdown items in a selectable, copyable view.
///
/// Text in paragraphs and headings can be selected with the mouse and copied
/// with Ctrl+C / Cmd+C. Use [`SelectableMarkdown::highlight_str`] to also
/// highlight search patterns before converting to [`Element`].
pub fn view_selectable<Message, Renderer>(
    items: impl IntoIterator<Item = Item<Message>>,
    settings: impl Into<Settings>,
) -> SelectableMarkdown<Message, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = Font>,
{
    SelectableMarkdown {
        items: items.into_iter().collect(),
        settings: settings.into(),
        highlights: Vec::new(),
        _message: std::marker::PhantomData,
        _renderer: std::marker::PhantomData,
    }
}

impl<Message, Renderer> SelectableMarkdown<Message, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = Font>,
{
    /// Highlight every occurrence of `pattern` using the given color function.
    /// Silently does nothing if `pattern` is empty.
    pub fn highlight_str(
        mut self,
        pattern: impl Into<String>,
        color: impl Fn(&Theme) -> Color + 'static,
    ) -> Self {
        let pattern = pattern.into();
        if !pattern.is_empty() {
            self.highlights.push((pattern, Arc::new(color)));
        }
        self
    }
}

fn collect_item_spans<Message>(
    it: &Item<Message>,
    settings: Settings,
    out: &mut Vec<text::Span<'static, Message>>,
) where
    Message: Clone,
{
    let nl = || -> text::Span<'static, Message> { span("\n") };

    match it {
        Item::Heading(level, t) => {
            let size = match level {
                HeadingLevel::H1 => settings.h1_size,
                HeadingLevel::H2 => settings.h2_size,
                HeadingLevel::H3 => settings.h3_size,
                HeadingLevel::H4 => settings.h4_size,
                HeadingLevel::H5 => settings.h5_size,
                HeadingLevel::H6 => settings.h6_size,
            };
            for s in t.spans(settings.style).iter() {
                out.push(s.clone().size(size));
            }
            out.push(nl());
        }
        Item::Paragraph(t) => {
            for s in t.spans(settings.style).iter() {
                out.push(s.clone());
            }
            out.push(nl());
        }
        Item::CodeBlock(code) => {
            out.push(span(code.clone()).font(settings.style.code_block_font));
            out.push(nl());
        }
        Item::List { start, bullets } => {
            for (i, bullet) in bullets.iter().enumerate() {
                match start {
                    None => out.push(span("• ")),
                    Some(n) => out.push(span(format!("{}. ", n + i as u64))),
                }
                for sub in bullet.items() {
                    collect_item_spans(sub, settings, out);
                }
            }
        }
        Item::Rule => {
            out.push(span("───────────────────────────────────────────\n"));
        }
    }
}

impl<'a, Message, Renderer> From<SelectableMarkdown<Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Paragraph = ConcreteP, Font = Font> + 'a,
    Message: 'static + Clone,
{
    fn from(md: SelectableMarkdown<Message, Renderer>) -> Self {
        let settings = md.settings;
        let mut all_spans: Vec<text::Span<'static, Message>> = Vec::new();
        let mut rule_lines: Vec<usize> = Vec::new();
        let mut heading_lines: Vec<usize> = Vec::new();

        let mut prev_was_heading = false;
        for (i, it) in md.items.iter().enumerate() {
            if i > 0 && !prev_was_heading {
                all_spans.push(span("\n"));
            }

            if matches!(it, Item::Heading(..)) {
                let heading_line = all_spans
                    .iter()
                    .map(|s| s.text.chars().filter(|&c| c == '\n').count())
                    .sum();
                heading_lines.push(heading_line);
            }

            collect_item_spans(it, settings, &mut all_spans);
            prev_was_heading = matches!(it, Item::Heading(..));
            if prev_was_heading {
                // The next line index = number of '\n' chars pushed so far.
                // Insert an empty line for the rule to occupy.
                let rule_line: usize = all_spans
                    .iter()
                    .map(|s| s.text.chars().filter(|&c| c == '\n').count())
                    .sum();
                rule_lines.push(rule_line);
                all_spans.push(span("\n"));
            }
        }

        let spans_arc: Arc<[text::Span<'static, Message>]> = all_spans.into();
        let mut srt = SelectableRichText::new(spans_arc, settings.text_size, settings.style.font)
            .with_rule_lines(rule_lines)
            .with_heading_lines(heading_lines);
        for (pattern, color_fn) in md.highlights {
            srt = srt.highlight_str_arc(pattern, color_fn);
        }
        Element::from(srt)
    }
}
