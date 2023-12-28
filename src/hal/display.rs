use crate::peripherals::mailbox;

pub struct EdidIterator {
    block_num: u32,
    done: bool
}

impl EdidIterator {
    pub fn new() -> Self {
        Self {block_num: 0, done: false}
    }
}

impl core::iter::Iterator for EdidIterator {
    type Item = (u32, [u8;128]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        use mailbox::PropertyMessageRequest::*;
        let mut mb = mailbox::Mailbox::<256>::new();
        mb.push_tag(GetEdidBlock { block_number: self.block_num });
        mb.push_tag(Null);
        if mb.submit_messages(8).is_ok() {
            let (block_number, status, data): (u32, u32, [u8; 128]) = mb.pop_values();
            if status == 0 {
                self.block_num += 1;
            } else {
                self.done = true;
            }
            Some((block_number, data))
        } else {
            self.done = true;
            None
        }
    }
}