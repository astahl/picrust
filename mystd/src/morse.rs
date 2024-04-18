use crate::collections::{MutSliceable, Sliceable};

/// ITU Morse Code https://en.wikipedia.org/wiki/Morse_code 
/// http://www.itu.int/rec/R-REC-M.1677-1-200910-I/
/// 

#[derive(Clone, Copy)]
pub enum MorseSymbol {
    Dot,
    Dash,
    SymbolSep,
    LetterSep,
    WordSep,
}

impl MorseSymbol {
    const fn unit_len(self) -> u8 {
        match self {
            MorseSymbol::Dot => 1,
            MorseSymbol::Dash => 3,
            MorseSymbol::SymbolSep => 1,
            MorseSymbol::LetterSep => 3,
            MorseSymbol::WordSep => 7,
        }
    } 

    const fn is_separator(self) -> bool {
        matches!(self, MorseSymbol::LetterSep | MorseSymbol::SymbolSep | MorseSymbol::WordSep)
    }
}


pub enum MorseLetter {
    Symbolic(char),
    Understood,
    Error,
    InvitationToTransmit,
    Wait,
    EndOfWork,
    StartingSignal
}

pub struct MorseText<S: Sliceable<MorseSymbol>> {
    buffer: S,
    len: usize,
}


impl<S: Sliceable<MorseSymbol>> MorseText<S> {
    pub const fn adapting(buffer: S) -> Self {
        Self { buffer, len: 0 }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }

    pub fn to_bools<'a>(&'a self) -> impl Iterator<Item = bool> + 'a {
        self.buffer.as_slice()[..self.len]
            .iter()
            .flat_map(|s| 
                core::iter::repeat(!s.is_separator())
                    .take(s.unit_len() as usize))
    }

}


impl<S: MutSliceable<MorseSymbol>> MorseText<S> {

    fn append_symbol(&mut self, symbol: MorseSymbol) {
        self.buffer.as_mut_slice()[self.len] = symbol;
        self.len += 1;
    }

    fn append_symbols(&mut self, symbols: &[MorseSymbol]) {
        let mut first = true;
        for s in symbols {
            if !first {
                self.append_symbol(MorseSymbol::SymbolSep);
            } else {
                first = false;
            }
            self.append_symbol(*s);
        }
    }

    pub fn put_letter(&mut self, l: MorseLetter) {
        use MorseSymbol::*;
        match l {
            MorseLetter::Symbolic(c) => match c {
                'A' | 'a' => self.append_symbols(&[Dot, Dash]),
                'B' | 'b' => self.append_symbols(&[Dash, Dot, Dot, Dot]),
                'C' | 'c' => self.append_symbols(&[Dash, Dot, Dash, Dot]),
                'D' | 'd' => self.append_symbols(&[Dash, Dot]),
                'E' | 'e' => self.append_symbols(&[Dot]),
                'F' | 'f' => self.append_symbols(&[Dot, Dot, Dash, Dot]),
                'G' | 'g' => self.append_symbols(&[Dash, Dash, Dot]),
                'H' | 'h' => self.append_symbols(&[Dot, Dot, Dot, Dot]),
                'I' | 'i' => self.append_symbols(&[Dot, Dot]),
                'J' | 'j' => self.append_symbols(&[Dot, Dash, Dash, Dash]),
                'K' | 'k' => self.append_symbols(&[Dash, Dot, Dash]),
                'L' | 'l' => self.append_symbols(&[Dot, Dash, Dot, Dot]),
                'M' | 'm' => self.append_symbols(&[Dash, Dash]),
                'N' | 'n' => self.append_symbols(&[Dash, Dot]),
                'O' | 'o' => self.append_symbols(&[Dash, Dash, Dash]),
                'P' | 'p' => self.append_symbols(&[Dot, Dash, Dash, Dot]),
                'Q' | 'q' => self.append_symbols(&[Dash, Dash, Dot, Dash]),
                'R' | 'r' => self.append_symbols(&[Dot, Dash, Dot]),
                'S' | 's' => self.append_symbols(&[Dot, Dot, Dot]),
                'T' | 't' => self.append_symbols(&[Dash]),
                'U' | 'u' => self.append_symbols(&[Dot, Dot, Dash]),
                'V' | 'v' => self.append_symbols(&[Dot, Dot, Dot, Dash]),
                'W' | 'w' => self.append_symbols(&[Dot, Dash, Dash]),
                'X' | 'x' => self.append_symbols(&[Dash, Dot, Dot, Dash]),
                'Y' | 'y' => self.append_symbols(&[Dash, Dot, Dash, Dash]),
                'Z' | 'z' => self.append_symbols(&[Dash, Dash, Dot, Dot]),
                '1' => self.append_symbols(&[Dot, Dash, Dash, Dash, Dash]),
                '2' => self.append_symbols(&[Dot, Dot, Dash, Dash, Dash]),
                '3' => self.append_symbols(&[Dot, Dot, Dot, Dash, Dash]),
                '4' => self.append_symbols(&[Dot, Dot, Dot, Dot, Dash]),
                '5' => self.append_symbols(&[Dot, Dot, Dot, Dot, Dot]),
                '6' => self.append_symbols(&[Dash, Dot, Dot, Dot, Dot]),
                '7' => self.append_symbols(&[Dash, Dash, Dot, Dot, Dot]),
                '8' => self.append_symbols(&[Dash, Dash, Dash, Dot, Dot]),
                '9' => self.append_symbols(&[Dash, Dash, Dash, Dash, Dot]),
                '0' => self.append_symbols(&[Dash, Dash, Dash, Dash, Dash]),
                '.' => self.append_symbols(&[Dot, Dash, Dot, Dash, Dot, Dash]),
                ',' => self.append_symbols(&[Dash, Dash, Dot, Dot, Dash, Dash]),
                ':' => self.append_symbols(&[Dash, Dash, Dash, Dot, Dot, Dot]),
                '?' => self.append_symbols(&[Dot, Dot, Dash, Dash, Dot, Dot]),
                '\'' | '`' | '´' => self.append_symbols(&[Dot, Dash, Dash, Dash, Dash, Dot]),
                '-' => self.append_symbols(&[Dash, Dot, Dot, Dot, Dot, Dash]),
                '/' => self.append_symbols(&[Dash, Dot, Dot, Dash, Dot]),
                '(' => self.append_symbols(&[Dash, Dot, Dash, Dash, Dot]),
                ')' => self.append_symbols(&[Dash, Dot, Dash, Dash, Dot, Dash]),
                '"' | '“' | '”' => self.append_symbols(&[Dot, Dash, Dot, Dot, Dash, Dot]),
                '=' => self.append_symbols(&[Dash, Dot, Dot, Dot, Dash]),
                '+' => self.append_symbols(&[Dot, Dash, Dot, Dash, Dot]),
                '*' | '×' => self.append_symbols(&[Dash, Dot, Dot, Dash]),
                '@' => self.append_symbols(&[Dot, Dash, Dash, Dot, Dash, Dot]),
                _ => {}
            }
            MorseLetter::Understood => self.append_symbols(&[Dot, Dot, Dot, Dash, Dot]),
            MorseLetter::Error => self.append_symbols(&[Dot, Dot, Dot, Dot, Dot, Dot, Dot, Dot]),
            MorseLetter::InvitationToTransmit => self.append_symbols(&[Dash, Dot, Dash]),
            MorseLetter::Wait => self.append_symbols(&[Dot, Dash, Dot, Dot, Dot]),
            MorseLetter::EndOfWork => self.append_symbols(&[Dot, Dot, Dot, Dash, Dot, Dash]),
            MorseLetter::StartingSignal => self.append_symbols(&[Dash, Dot, Dash, Dot, Dash]),
        }
    }

    pub fn write_str(&mut self, s: &str) {
        let mut first_word = true;
        for word in s.split_whitespace() {
            if first_word {
                first_word = false;
            } else {
                self.append_symbol(MorseSymbol::WordSep);
            }

            let mut first_letter = true;
            for c in word.chars() {
                if first_letter {
                    first_letter = false;
                } else {
                    self.append_symbol(MorseSymbol::LetterSep)
                }
                self.put_letter(MorseLetter::Symbolic(c))
            }
        }
    }

}

pub type MorseTextArray<const N: usize> = MorseText<[MorseSymbol;N]>; 
pub type MorseTextSlice<'a> = MorseText<&'a [MorseSymbol]>; 
pub type MorseTextMutSlice<'a> = MorseText<&'a mut [MorseSymbol]>; 

impl<const N: usize> MorseTextArray<N> {
    pub fn new() -> Self {
        Self::adapting([MorseSymbol::WordSep; N])
    }

    pub fn as_slice(&self) -> MorseTextSlice {
        MorseText::adapting(&self.buffer)
    }

    pub fn as_mut_slice(&mut self) -> MorseTextMutSlice {
        MorseText::adapting(&mut self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let mut text: MorseTextArray<200> = MorseTextArray::new();
        text.write_str("Sos :)");
        let mut bools = text.to_bools();
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());        
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
        assert_eq!(Some(true), bools.next());
        assert_eq!(Some(false), bools.next());
    }
}