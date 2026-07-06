use crate::text::{byte_index, char_len, compose_hangul_jamo, display_width, prefix};
use ratatui::layout::{Position, Rect};

#[derive(Clone, Debug)]
pub struct TextEditor {
    lines: Vec<String>,
    row: usize,
    col: usize,
    scroll: usize,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            row: 0,
            col: 0,
            scroll: 0,
        }
    }
}

impl TextEditor {
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.split('\n').map(str::to_string).collect();
        if text.ends_with('\n') {
            self.lines.pop();
            self.lines.push(String::new());
        }
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = 0;
        self.col = 0;
        self.scroll = 0;
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub(super) fn visible_text(&mut self, height: usize) -> String {
        if self.row < self.scroll {
            self.scroll = self.row;
        } else if height > 0 && self.row >= self.scroll + height {
            self.scroll = self.row + 1 - height;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        self.lines
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(height.max(1))
            .map(|(index, line)| {
                let cursor = if index == self.row { ">" } else { " " };
                format!("{cursor}{:>width$} {line}", index + 1, width = line_width)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn cursor_position(&self, area: Rect) -> Option<Position> {
        if self.row < self.scroll {
            return None;
        }
        let visible_row = self.row - self.scroll;
        let inner_height = area.height.saturating_sub(2) as usize;
        if visible_row >= inner_height {
            return None;
        }
        let line_width = ((self.lines.len().max(1)).to_string().len()).max(3);
        let prefix_width = 1 + line_width + 1;
        let line = self.lines.get(self.row)?;
        let text_before_cursor = prefix(line, self.col);
        let x = area
            .x
            .saturating_add(1)
            .saturating_add((prefix_width + display_width(&text_before_cursor)) as u16)
            .min(area.right().saturating_sub(2));
        let y = area.y.saturating_add(1).saturating_add(visible_row as u16);
        Some(Position::new(x, y))
    }

    pub fn insert_char(&mut self, char: char) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        self.lines[self.row].insert(byte, char);
        self.col += 1;
        self.normalize_current_line();
    }

    pub fn insert_newline(&mut self) {
        self.ensure_cursor();
        let byte = byte_index(&self.lines[self.row], self.col);
        let rest = self.lines[self.row].split_off(byte);
        self.lines.insert(self.row + 1, rest);
        self.row += 1;
        self.col = 0;
    }

    pub fn backspace(&mut self) {
        self.ensure_cursor();
        if self.col > 0 {
            let start = byte_index(&self.lines[self.row], self.col - 1);
            let end = byte_index(&self.lines[self.row], self.col);
            self.lines[self.row].replace_range(start..end, "");
            self.col -= 1;
            self.normalize_current_line();
        } else if self.row > 0 {
            let current = self.lines.remove(self.row);
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
            self.lines[self.row].push_str(&current);
        }
    }

    pub(super) fn delete(&mut self) {
        self.ensure_cursor();
        if self.col < char_len(&self.lines[self.row]) {
            let start = byte_index(&self.lines[self.row], self.col);
            let end = byte_index(&self.lines[self.row], self.col + 1);
            self.lines[self.row].replace_range(start..end, "");
            self.normalize_current_line();
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        }
    }

    pub(super) fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = char_len(&self.lines[self.row]);
        }
    }

    pub(super) fn move_right(&mut self) {
        if self.col < char_len(&self.lines[self.row]) {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    pub(super) fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    pub(super) fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(char_len(&self.lines[self.row]));
        }
    }

    pub(super) fn move_page_up(&mut self, height: usize) {
        self.row = self.row.saturating_sub(height.max(1));
        self.col = self.col.min(char_len(&self.lines[self.row]));
    }

    pub(super) fn move_page_down(&mut self, height: usize) {
        self.row = (self.row + height.max(1)).min(self.lines.len().saturating_sub(1));
        self.col = self.col.min(char_len(&self.lines[self.row]));
    }

    fn ensure_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.row = self.row.min(self.lines.len() - 1);
        self.col = self.col.min(char_len(&self.lines[self.row]));
    }

    fn normalize_current_line(&mut self) {
        let normalized = compose_hangul_jamo(&self.lines[self.row]);
        if normalized == self.lines[self.row] {
            self.col = self.col.min(char_len(&self.lines[self.row]));
            return;
        }
        let old_prefix = prefix(&self.lines[self.row], self.col);
        self.lines[self.row] = normalized;
        self.col = char_len(&compose_hangul_jamo(&old_prefix)).min(char_len(&self.lines[self.row]));
    }
}
