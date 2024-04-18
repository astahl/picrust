use mystd::slice::slice2d::{traits::MutSlice2dTrait, MutSlice2d};

use crate::{println_debug, system::{self, output::std_out}};

pub struct Console<'a> {
    tile_map: MutSlice2d<'a, u8>,
    line: u8,
}

impl Console<'_> {
    pub fn new(base_ptr: *mut u8) -> Self {
        let tile_height = 8;
        let tile_width = 8;
        let screen = system::screen::shared();
        let (cols, rows) = screen.lock().with_screen_mut(|s| (s.width() / tile_width, s.height() / tile_height)).unwrap_or_default();
        Self { 
            tile_map: unsafe { MutSlice2d::from_raw_parts(base_ptr, cols, cols.next_power_of_two(), rows) },
            line: 3
        }
    }
}

impl<'a> mystd::io::Write for Console<'a> {
    fn write(&mut self, buf: &[u8]) -> mystd::io::Result<mystd::io::Size> {
        let mut locked_screen = system::screen::shared().lock();
        locked_screen.with_screen_mut(|s| {
            s.draw(
                |buffer| {
                    buffer.sub_mut_slice2d((.., self.line as usize..self.line as usize + 1)).fill(self.line);
                });
            s.present(system::screen::SwapStrategy::Swap, system::screen::PresentStrategy::Memcopy);
        });
        self.line = self.line.wrapping_add(1);
        Ok(mystd::io::Size::from_usize(buf.len()))
    }

    fn flush(&mut self) -> mystd::io::Result<()> {
        Ok(())
    }
}