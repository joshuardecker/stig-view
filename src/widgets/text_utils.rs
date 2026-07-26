use cosmic_text::{Buffer, BufferLine, LayoutRun};
use iced::Point;
use iced::advanced::mouse::{Click, click};
use unicode_segmentation::UnicodeSegmentation as _;

/// Like `buffer.hit(x, y)` but falls back to a y-based line lookup when `hit()` returns
/// `None`. This handles blank lines, which have no glyphs and thus can't be hit directly.
/// Returns `(logical_line_index, byte_offset)`.
pub fn hit_or_nearest(buffer: &Buffer, x: f32, y: f32) -> Option<(usize, usize)> {
    if let Some(cursor) = buffer.hit(x, y) {
        return Some((cursor.line, cursor.index));
    }

    if buffer.lines.is_empty() {
        return None;
    }

    let line_height = buffer.metrics().line_height;

    if line_height <= 0.0 {
        return None;
    }

    let target_visual = (y / line_height).max(0.0) as usize;

    let mut visual_start = 0usize;

    for (line_idx, buffer_line) in buffer.lines.iter().enumerate() {
        let visual_count = buffer_line
            .layout_opt()
            .map(|layout| layout.len())
            .unwrap_or(1)
            .max(1);

        if target_visual < visual_start + visual_count || line_idx + 1 == buffer.lines.len() {
            return Some((line_idx, buffer_line.text().len()));
        }

        visual_start += visual_count;
    }

    let last = buffer.lines.len() - 1;

    Some((last, buffer.lines[last].text().len()))
}

/// Returns `(x, width)` of the highlighted byte range `[from, to)` within a slice of glyphs.
pub(crate) fn highlight_glyphs(
    glyphs: &[cosmic_text::LayoutGlyph],
    from: usize,
    to: usize,
) -> (f32, f32) {
    if glyphs.is_empty() {
        return (0.0, 0.0);
    }

    let line_start = glyphs.first().map(|glyph| glyph.start).unwrap_or(0);

    let line_end = glyphs.last().map(|glyph| glyph.end).unwrap_or(0);

    let range = line_start.max(from)..line_end.min(to);

    if range.is_empty() {
        return (0.0, 0.0);
    }

    let first = glyphs
        .iter()
        .position(|glyph| range.start <= glyph.start)
        .unwrap_or(0);

    let mut glyphs_iter = glyphs.iter();

    let x_pos: f32 = glyphs_iter.by_ref().take(first).map(|glyph| glyph.w).sum();

    let width: f32 = glyphs_iter
        .take_while(|glyph| range.end > glyph.start)
        .map(|glyph| glyph.w)
        .sum();

    (x_pos, width)
}

/// Returns `(x, width)` for each visual sub-line within a `BufferLine`.
pub fn highlight_line(buffer_line: &BufferLine, from: usize, to: usize) -> Vec<(f32, f32)> {
    let layout = buffer_line
        .layout_opt()
        .map(|layout_vec| layout_vec.as_slice())
        .unwrap_or(&[]);

    layout
        .iter()
        .map(|visual_line| highlight_glyphs(&visual_line.glyphs, from, to))
        .collect()
}

/// Returns `(x, width)` of the highlighted byte range within a `LayoutRun`.
pub fn highlight_run(run: &LayoutRun<'_>, from: usize, to: usize) -> (f32, f32) {
    highlight_glyphs(run.glyphs, from, to)
}

/// Normalize a selection range so that start <= end.
pub fn normalize_selection(
    anchor_line: usize,
    anchor_idx: usize,
    focus_line: usize,
    focus_idx: usize,
) -> ((usize, usize), (usize, usize)) {
    if (anchor_line, anchor_idx) <= (focus_line, focus_idx) {
        ((anchor_line, anchor_idx), (focus_line, focus_idx))
    } else {
        ((focus_line, focus_idx), (anchor_line, anchor_idx))
    }
}

/// Extract the selected text from a buffer given a selection range.
pub fn extract_selection_text(
    buffer: &Buffer,
    anchor_line: usize,
    anchor_idx: usize,
    focus_line: usize,
    focus_idx: usize,
) -> Option<String> {
    let ((start_line, start_idx), (end_line, end_idx)) =
        normalize_selection(anchor_line, anchor_idx, focus_line, focus_idx);

    if (start_line, start_idx) >= (end_line, end_idx) {
        return None;
    }

    let mut selected_text = String::new();

    let selected_logical_lines = end_line - start_line + 1;

    for (line_idx, buffer_line) in buffer
        .lines
        .iter()
        .skip(start_line)
        .take(selected_logical_lines)
        .enumerate()
    {
        if line_idx > 0 {
            selected_text.push('\n');
        }

        let text = buffer_line.text();

        let from = if line_idx == 0 { start_idx } else { 0 };

        let to = if line_idx == selected_logical_lines - 1 {
            end_idx
        } else {
            text.len()
        };

        selected_text.push_str(&text[from.min(text.len())..to.min(text.len())]);
    }

    if selected_text.is_empty() {
        None
    } else {
        Some(selected_text)
    }
}

/// Compute a new selection based on a mouse click inside a text buffer.
///
/// Returns `None` if the click does not hit any text (e.g., empty padding).
pub fn selection_from_click(
    buffer: &Buffer,
    click: Click,
    mouse_pos: Point,
) -> Option<((usize, usize), (usize, usize))> {
    let cursor = buffer.hit(mouse_pos.x, mouse_pos.y)?;

    let line_text = buffer.lines[cursor.line].text();

    Some(match click.kind() {
        click::Kind::Single => ((cursor.line, cursor.index), (cursor.line, cursor.index)),

        click::Kind::Double => {
            let start = line_text
                .unicode_word_indices()
                .rev()
                .map(|(byte_idx, _)| byte_idx)
                .find(|&byte_idx| byte_idx < cursor.index)
                .unwrap_or(0);

            let end = line_text
                .unicode_word_indices()
                .map(|(byte_idx, word)| byte_idx + word.len())
                .find(|&byte_idx| byte_idx > cursor.index)
                .unwrap_or(line_text.len());

            ((cursor.line, start), (cursor.line, end))
        }

        click::Kind::Triple => ((cursor.line, 0), (cursor.line, line_text.len())),
    })
}

/// Compute a new selection while dragging.
///
/// Returns `None` if the drag position does not resolve to a text position.
pub fn selection_from_drag(
    buffer: &Buffer,
    anchor: (usize, usize),
    mouse_pos: Point,
) -> Option<((usize, usize), (usize, usize))> {
    let focus = hit_or_nearest(buffer, mouse_pos.x, mouse_pos.y)?;

    Some((anchor, focus))
}
