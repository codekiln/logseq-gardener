//! Where each line of a document starts, for turning an offset into a line and column.

/// A line number and a column, both counted from zero. The column is an offset into the
/// line's UTF-8 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinePosition {
    pub line: usize,
    pub column: usize,
}

/// The offset at which each line of a text starts. A line ends after its `\n`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    starts: Vec<usize>,
    length: usize,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            starts,
            length: text.len(),
        }
    }

    /// How many lines the text has; an empty text has one.
    pub fn line_count(&self) -> usize {
        self.starts.len()
    }

    /// The line and column of an offset. An offset past the end gives the end of the text.
    pub fn position(&self, offset: usize) -> LinePosition {
        let offset = offset.min(self.length);
        let line = self.starts.partition_point(|start| *start <= offset) - 1;
        LinePosition {
            line,
            column: offset - self.starts[line],
        }
    }

    /// The offsets at which a line starts and ends, without its line ending. A line past
    /// the end gives the end of the text.
    pub fn line_range(&self, line: usize, text: &str) -> (usize, usize) {
        let Some(&start) = self.starts.get(line) else {
            return (self.length, self.length);
        };
        let end = self.starts.get(line + 1).copied().unwrap_or(self.length);
        let content = text.get(start..end).unwrap_or_default();
        let content = content.strip_suffix('\n').unwrap_or(content);
        let content = content.strip_suffix('\r').unwrap_or(content);
        (start, start + content.len())
    }

    /// The offset of a line and column. A column past the end of the line gives the end of
    /// the line, and a column inside a character gives the start of that character.
    pub fn offset(&self, position: LinePosition, text: &str) -> usize {
        let (start, end) = self.line_range(position.line, text);
        let mut offset = (start + position.column).min(end);
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_offsets_and_positions() {
        let text = "- a\n\t- é [[B]]\r\n\nlast";
        let lines = LineIndex::new(text);
        assert_eq!(lines.line_count(), 4);
        let link = text.find("[[B]]").unwrap();
        let position = lines.position(link);
        assert_eq!(position, LinePosition { line: 1, column: 6 });
        assert_eq!(lines.offset(position, text), link);
        assert_eq!(lines.line_range(1, text), (4, link + 5));
        assert_eq!(lines.line_range(2, text), (link + 7, link + 7));
        assert_eq!(
            lines.position(text.len()),
            LinePosition { line: 3, column: 4 }
        );
    }

    #[test]
    fn keeps_out_of_range_positions_inside_the_text() {
        let text = "é\nb";
        let lines = LineIndex::new(text);
        let at = |line, column| lines.offset(LinePosition { line, column }, text);
        // Inside `é`, past the line, and past the text.
        assert_eq!(at(0, 1), 0);
        assert_eq!(at(0, 9), 2);
        assert_eq!(at(7, 0), text.len());
        assert_eq!(lines.position(99), LinePosition { line: 1, column: 1 });
    }
}
