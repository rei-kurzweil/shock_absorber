#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatEditor {
    text: String,
    cursor: usize,
}

impl ChatEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn trimmed_text(&self) -> &str {
        self.text.trim()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn take_text(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.text)
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn backspace(&mut self) {
        let Some((start, _)) = previous_grapheme_boundary(&self.text, self.cursor) else {
            return;
        };
        self.text.drain(start..self.cursor);
        self.cursor = start;
    }

    pub fn move_left(&mut self) {
        if let Some((start, _)) = previous_grapheme_boundary(&self.text, self.cursor) {
            self.cursor = start;
        }
    }

    pub fn move_right(&mut self) {
        if let Some((_, end)) = next_grapheme_boundary(&self.text, self.cursor) {
            self.cursor = end;
        }
    }

    pub fn move_up(&mut self) {
        let (line_index, column) = self.cursor_line_and_column();
        if line_index == 0 {
            return;
        }
        let line_starts = self.line_starts();
        let target_start = line_starts[line_index - 1];
        let target_end = line_end(&self.text, target_start);
        self.cursor = advance_columns(&self.text, target_start, target_end, column);
    }

    pub fn move_down(&mut self) {
        let line_starts = self.line_starts();
        let (line_index, column) = self.cursor_line_and_column();
        if line_index + 1 >= line_starts.len() {
            return;
        }
        let target_start = line_starts[line_index + 1];
        let target_end = line_end(&self.text, target_start);
        self.cursor = advance_columns(&self.text, target_start, target_end, column);
    }

    pub fn lines(&self) -> Vec<&str> {
        self.text.split('\n').collect()
    }

    pub fn cursor_line_and_column(&self) -> (usize, usize) {
        let mut line = 0;
        let mut column = 0;
        for (idx, ch) in self.text.char_indices() {
            if idx >= self.cursor {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0];
        for (idx, ch) in self.text.char_indices() {
            if ch == '\n' {
                starts.push(idx + ch.len_utf8());
            }
        }
        starts
    }
}

fn previous_grapheme_boundary(text: &str, cursor: usize) -> Option<(usize, usize)> {
    if cursor == 0 {
        return None;
    }
    let mut iter = text[..cursor].char_indices();
    let (start, ch) = iter.next_back()?;
    Some((start, start + ch.len_utf8()))
}

fn next_grapheme_boundary(text: &str, cursor: usize) -> Option<(usize, usize)> {
    if cursor >= text.len() {
        return None;
    }
    let mut iter = text[cursor..].char_indices();
    let (offset, ch) = iter.next()?;
    let start = cursor + offset;
    Some((start, start + ch.len_utf8()))
}

fn line_end(text: &str, start: usize) -> usize {
    text[start..]
        .find('\n')
        .map(|offset| start + offset)
        .unwrap_or(text.len())
}

fn advance_columns(text: &str, start: usize, end: usize, columns: usize) -> usize {
    let mut cursor = start;
    let mut consumed = 0;
    for (offset, ch) in text[start..end].char_indices() {
        if consumed >= columns {
            break;
        }
        cursor = start + offset + ch.len_utf8();
        consumed += 1;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::ChatEditor;

    #[test]
    fn inserts_in_middle_after_vertical_move() {
        let mut editor = ChatEditor::new();
        for ch in "alpha\nbeta".chars() {
            editor.insert_char(ch);
        }

        editor.move_up();
        editor.move_left();
        editor.insert_char('X');

        assert_eq!(editor.text(), "alpXha\nbeta");
    }

    #[test]
    fn backspace_removes_character_before_cursor() {
        let mut editor = ChatEditor::new();
        for ch in "abc".chars() {
            editor.insert_char(ch);
        }

        editor.move_left();
        editor.backspace();

        assert_eq!(editor.text(), "ac");
    }

    #[test]
    fn moves_down_preserving_column_when_possible() {
        let mut editor = ChatEditor::new();
        for ch in "abc\nxyz".chars() {
            editor.insert_char(ch);
        }

        editor.move_up();
        editor.move_down();

        assert_eq!(editor.cursor_line_and_column(), (1, 3));
    }
}
