

pub trait Index2d {
    fn x(&self) -> usize;
    fn y(&self) -> usize;
}

impl Index2d for (usize, usize) {
    fn x(&self) -> usize {
        self.0.into()
    }
    fn y(&self) -> usize {
        self.1.into()
    }
}

impl Index2d for &(usize, usize) {
    fn x(&self) -> usize {
        self.0.into()
    }
    fn y(&self) -> usize {
        self.1.into()
    }
}

impl Index2d for [usize;2] {
    fn x(&self) -> usize {
        self[0].into()
    }
    fn y(&self) -> usize {
        self[1].into()
    }
}

impl Index2d for &[usize;2] {
    fn x(&self) -> usize {
        self[0].into()
    }
    fn y(&self) -> usize {
        self[1].into()
    }
}
