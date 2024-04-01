use crate::slice::slice2d::traits::{MutSlice2dTrait, Slice2dTrait};

pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

pub enum TextDirection {
    LeftToRight,
    RightToLeft
}

pub enum LineWrapping {
    None,
    Auto(usize)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakType {
    LastLine,
    LineWrap,
    WordWrap,
    CarriageReturn,
    NewLine,
}

struct AutoLineWrapper<'a> {
    text: &'a str,
    max_line_length: usize,
}

impl<'a> Iterator for AutoLineWrapper<'a> {
    type Item = (BreakType, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            None
        } else {
            let mut remaining_length = self.max_line_length;
            let mut split_point: usize = 0;
            let mut current_word_start: Option<usize> = None;
            let mut last_ws_start: Option<usize> = None;
            let mut skip: usize = 0;
            let mut iter = self.text.char_indices().peekable();
            let break_type = loop {
                if let Some((pos, c)) = iter.next() {
                    if c == '\n' {
                        if remaining_length == 0 {
                            split_point = last_ws_start.unwrap_or_default();
                            skip = 1 + pos - last_ws_start.unwrap_or_default();
                        } else {
                            split_point = pos;
                            skip = 1;
                        }
                        break BreakType::NewLine;
                    }
                    if c == '\r' {
                        // this might be the first char in a \r\n chord
                        if remaining_length == 0 {
                            split_point = last_ws_start.unwrap_or_default();
                            skip = pos - last_ws_start.unwrap_or_default();
                        } else {
                            split_point = pos;
                        }
                        if let Some((_, nc)) = iter.peek() {
                            if *nc == '\n' {
                                skip += 2;
                                break BreakType::NewLine;
                            }
                        }
                        skip += 1;
                        // handle as a carriage return
                        break BreakType::CarriageReturn;
                    }
                    
                    if c.is_whitespace() {
                        if current_word_start.is_some() {
                            current_word_start = None;
                            last_ws_start = Some(pos);
                        } 
                    } else {
                        if current_word_start.is_none() {
                            current_word_start = Some(pos)
                        }
                        if remaining_length == 0 {
                            if last_ws_start.is_none() {
                                split_point = pos;
                                skip = 0;
                                break BreakType::WordWrap;
                                // if let Some((_, nc)) = iter.peek() {
                                //     if !nc.is_whitespace() {
                                //     }
                                // }
                            } else {
                                split_point = last_ws_start.unwrap_or_default();
                                skip = current_word_start.unwrap_or_default() - last_ws_start.unwrap_or_default();
                                break BreakType::LineWrap;
                            }
                        } else {
                            
                        }
                    }
                    remaining_length = remaining_length.saturating_sub(1);
                } else {
                    split_point = self.text.len();
                    break BreakType::LastLine;
                }
            };
            let (line, remaining) = self.text.split_at(split_point);
            self.text = unsafe { remaining.get_unchecked(skip..) };
            Some((break_type, line))
        }
    }
}

impl<'a> AutoLineWrapper<'a> {
    fn new(text: &'a str, max_line_length: usize) -> Self {
        Self {
            text,
            max_line_length,
        }
    }
}


pub struct Formatter {
    direction: TextDirection,
    wrapping: LineWrapping,
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
}

impl Formatter {


    pub fn write_to_slice2d<'t, 'a, T, M: Fn(char) -> T>(&self, text: &'t str, character_mapping: M, slice2d: &'a mut crate::slice::slice2d::MutSlice2d<T>) -> &'t str {
        let max_line_length = slice2d.width();
        let wrapped_lines = AutoLineWrapper::new(text, max_line_length);
        let mut last_line_index: usize = 0;
        for (src_line, dst_line) in wrapped_lines.zip(slice2d.rows_mut()) {
            for (src_char, dst_char) in src_line.1.chars().map(&character_mapping).zip(dst_line) {
                *dst_char = src_char;
            }
            last_line_index = unsafe {
                src_line.1.as_ptr().offset_from(text.as_ptr()) 
            } as usize + src_line.1.len(); 
        }
        text.split_at(last_line_index).1
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_line_wrapper_works() {

        let line = "This is a line of    text with somehighlyinteresting content.  \nBut? Why?     \r\nSo";
        let mut iter = AutoLineWrapper::new(line, 10);
        assert_eq!(iter.next(), Some((BreakType::LineWrap, "This is a")));
        assert_eq!(iter.next(), Some((BreakType::LineWrap, "line of")));
        assert_eq!(iter.next(), Some((BreakType::LineWrap, "text with")));
        assert_eq!(iter.next(), Some((BreakType::WordWrap, "somehighly")));
        assert_eq!(iter.next(), Some((BreakType::WordWrap, "interestin")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "g content.")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "But? Why?")));
        assert_eq!(iter.next(), Some((BreakType::LastLine, "So")));
        
    }

    #[test]
    fn auto_line_wrapper_handles_newline() {

        let line = "This is a line\n of text \n  with\n\n some highly interesting content.";
        let mut iter = AutoLineWrapper::new(line, 80);
        assert_eq!(iter.next(), Some((BreakType::NewLine, "This is a line")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, " of text ")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "  with")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "")));
        assert_eq!(iter.next(), Some((BreakType::LastLine, " some highly interesting content.")));
        assert_eq!(iter.next(), None);
        
    }

    #[test]
    fn auto_line_wrapper_handles_carriage_return() {

        let line = "This is a line\n of text \r\n  with\n\n some \rhighly interesting content.";
        let mut iter = AutoLineWrapper::new(line, 80);
        assert_eq!(iter.next(), Some((BreakType::NewLine, "This is a line")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, " of text ")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "  with")));
        assert_eq!(iter.next(), Some((BreakType::NewLine, "")));
        assert_eq!(iter.next(), Some((BreakType::CarriageReturn, " some ")));
        assert_eq!(iter.next(), Some((BreakType::LastLine, "highly interesting content.")));
        
    }
}