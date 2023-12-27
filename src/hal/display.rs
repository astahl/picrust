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
        let request = [
            GetEdidBlock { block_number: self.block_num },
            Null
        ];
        if let Ok(response) = mb.request(8, &request) {

            use mailbox::PropertyMessageResponse::*;
            match response.first() {
                Some(GetEdidBlock { block_number, status, data }) => {
                    //crate::peripherals::uart::Uart0::put_hex_bytes(data);
                    if *status == 0 {
                        self.block_num += 1;
                    } else {
                        self.done = true;
                    }
                    Some((*block_number, *data))
                },
                _ => {
                    self.done = true;
                    None
                }
            }
        } else {
            self.done = true;
            None
        }
    }
}