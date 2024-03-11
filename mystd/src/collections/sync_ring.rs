use core::sync::atomic;
use atomic::Ordering::*;

pub struct AtomicRing256<T> {
    read_write_idx: atomic::AtomicU16,
    data: [T; 256],
}

impl<T> AtomicRing256<T> {
    pub fn new() -> Self where T: Default + Copy {
        Self { 
            read_write_idx: atomic::AtomicU16::new(0), 
            data: [Default::default(); 256] 
        }
    }

    pub fn is_empty(&self) -> bool {
        let [read, write] = self.read_write_idx.load(Relaxed).to_ne_bytes();
        read == write
    }

    pub fn is_full(&self) -> bool {
        let [read, write] = self.read_write_idx.load(Relaxed).to_ne_bytes();
        read == write.wrapping_add(1)
    }

    pub fn capacity(&self) -> usize {
        let [read, write] = self.read_write_idx.load(Relaxed).to_ne_bytes();
        read.wrapping_sub(write).wrapping_sub(1) as usize
    }

    pub fn put(&mut self, value: T) -> Result<T,()>{
        let index_update = self.read_write_idx.fetch_update(SeqCst, SeqCst, |current| {
            let [read, mut write] = current.to_ne_bytes();
            write = write.wrapping_add(1);
            if read == write {
                None
            } else {
                Some(u16::from_ne_bytes([read, write]))
            }
        });
        match index_update {
            Ok(old_index) => {
                let [_, write] = old_index.to_ne_bytes();
                let dest = unsafe { self.data.get_unchecked_mut(write as usize) };
                Ok(core::mem::replace(dest, value))
            },
            Err(_) => Err(()),
        }
    }

    /// Returns None when the buffer is empty
    pub fn pop_take(&mut self) -> Option<T> where T: Default {
        let index_update = self.read_write_idx.fetch_update(SeqCst, SeqCst, |current| {
            let [read, write] = current.to_ne_bytes();
            if read == write {
                None
            } else {
                Some(u16::from_ne_bytes([read.wrapping_add(1), write]))
            }
        });
        match index_update {
            Ok(old_index) => {
                let [read, _] = old_index.to_ne_bytes();
                let src = unsafe { self.data.get_unchecked_mut(read as usize) };
                Some(core::mem::take(src))
            },
            Err(_) => None,
        }
    }

    /// Returns None when the buffer is empty
    pub fn peek_copy(&mut self) -> Option<T> where T: Copy {
        let mut latest_copy: Option<T> = None;
        let index_update = self.read_write_idx.fetch_update(SeqCst, SeqCst, |current| {
            let [read, write] = current.to_ne_bytes();
            if read == write {
                None
            } else {
                latest_copy = Some(*unsafe { self.data.get_unchecked(read as usize) });
                Some(current)
            }
        });
        match index_update {
            Ok(_) => latest_copy,
            Err(_) => None,
        }
    }

    pub fn read_write_indexes(&self) -> (u8, u8) {
        self.read_write_idx.load(Relaxed).to_ne_bytes().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_empty_full_works() {
        let mut buf = AtomicRing256::<u8>::new();
        assert_eq!(255, buf.capacity());
        assert!(buf.is_empty());
        while buf.capacity() > 0 {
            assert!(buf.put(1).is_ok());
        }
        assert!(buf.is_full());
        assert_eq!((0, 255), buf.read_write_indexes());
        assert!(buf.put(2).is_err());
    }
}