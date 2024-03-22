pub struct ColIter<'a, T> {
    start: *const T,
    end: *const T,
    height: usize,
    pitch: usize,
    phantom_data: core::marker::PhantomData<&'a T>
}

pub struct RowIter<'a, T> {
    start_iter: ColIter<'a, T>,
    width: usize
}

pub struct ColIterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    height: usize,
    pitch: usize,
    phantom_data: core::marker::PhantomData<&'a T>
}

pub struct RowIterMut<'a, T> {
    start_iter: ColIterMut<'a, T>,
    width: usize,
}

impl<'a, T> ColIter<'a, T> {
    pub fn new(base: *const T, pitch: usize, height: usize) -> Self {
        Self {
            start: base.wrapping_sub(pitch),
            end: base.wrapping_add(pitch * height),
            height,
            pitch,
            phantom_data: core::marker::PhantomData,
        }
    }
}

impl<'a, T> RowIter<'a, T> {
    pub fn new(base: *const T, width: usize, pitch: usize, height: usize) -> Self {
        Self {
            start_iter: ColIter::new(base, pitch, height),
            width
        }
    }
}

impl<'a, T> ColIterMut<'a, T> {
    pub fn new(base: *mut T, pitch: usize, height: usize) -> Self {
        Self {
            start: base.wrapping_sub(pitch),
            end: base.wrapping_add(pitch * height),
            height,
            pitch,
            phantom_data: core::marker::PhantomData,
        }
    }
}

impl<'a, T> RowIterMut<'a, T> {
    pub fn new(base: *mut T, width: usize, pitch: usize, height: usize) -> Self {
        Self {
            start_iter: ColIterMut::new(base, pitch, height),
            width
        }
    }
}

impl<'a, T> Iterator for ColIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            self.start = self.start.wrapping_add(self.pitch);
            if self.start == self.end {
                None
            } else {
                self.height -= 1;
                Some(unsafe { &*self.start })
            }
        }
    }
}

impl<'a, T> Iterator for RowIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.start_iter.next().map(|s| unsafe { core::slice::from_raw_parts(s, self.width) })
    }
}

impl<'a, T> Iterator for ColIterMut<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            self.start = self.start.wrapping_add(self.pitch);
            if self.start == self.end {
                None
            } else {
                self.height -= 1;
                Some(unsafe { &*self.start })
            }
        }
    }
}

impl<'a, T> Iterator for RowIterMut<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.start_iter.next().map(|s| unsafe { core::slice::from_raw_parts(s, self.width) })
    }
}

impl<'a, T> DoubleEndedIterator for ColIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.start {
            None
        } else {
            self.end = self.end.wrapping_sub(self.pitch);
            if self.end == self.start {
                None
            } else {
                self.height -= 1;
                Some(unsafe { &*self.end })
            }
        }
    }
}

impl<'a, T> DoubleEndedIterator for RowIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.start_iter.next_back().map(|s| unsafe { core::slice::from_raw_parts(s, self.width) })
    }
}

impl<'a, T> DoubleEndedIterator for ColIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.start {
            None
        } else {
            self.end = self.end.wrapping_sub(self.pitch);
            if self.end == self.start {
                None
            } else {
                self.height -= 1;
                Some(unsafe { &*self.end })
            }
        }
    }
}

impl<'a, T> DoubleEndedIterator for RowIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.start_iter.next_back().map(|s| unsafe { core::slice::from_raw_parts(s, self.width) })
    }
}

impl<'a, T> ExactSizeIterator for ColIter<'a, T> {
    fn len(&self) -> usize {
        self.height
    }
}

impl<'a, T> ExactSizeIterator for RowIter<'a, T> {
    fn len(&self) -> usize {
        self.start_iter.len()
    }
}

impl<'a, T> ExactSizeIterator for ColIterMut<'a, T> {
    fn len(&self) -> usize {
        self.height
    }
}

impl<'a, T> ExactSizeIterator for RowIterMut<'a, T> {
    fn len(&self) -> usize {
        self.start_iter.len()
    }
}

