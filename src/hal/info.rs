use crate::peripherals::mailbox;

pub struct Memory {
    pub base_address: usize,
    pub size: usize,
}

pub struct BoardInfo {
    pub model: u32,
    pub revision: u32,
    pub serial: u64,
}

pub fn get_arm_memory() -> Option<Memory>{
    let mut mb = mailbox::Mailbox::<256>::new();
    use mailbox::PropertyMessageRequest::*;
    mb.push_tag(HwGetArmMemory);
    mb.push_tag(Null);
    if mb.submit_messages(8).is_ok() {
        let (base_address, size): (u32, u32) = mb.pop_values();
        Some(Memory{
            base_address: base_address as usize,
            size: size as usize
        })
    } else {
        None
    }
}

pub fn get_vc_memory() -> Option<Memory>{
    let mut mb = mailbox::Mailbox::<256>::new();
    use mailbox::PropertyMessageRequest::*;
    mb.push_tag(HwGetVcMemory);
    mb.push_tag(Null);
    if mb.submit_messages(8).is_ok() {
        let (base_address, size): (u32, u32) = mb.pop_values();
        Some(Memory{
            base_address: base_address as usize,
            size: size as usize
        })
    } else {
        None
    }
}

pub fn get_board_info() -> Option<BoardInfo>{
    use mailbox::PropertyMessageRequest::*;
    let mut mb = mailbox::Mailbox::<256>::new();
    mb.push_tag(HwGetBoardModel);
    mb.push_tag(HwGetBoardRevision);
    mb.push_tag(HwGetBoardSerial);
    mb.push_tag(Null);
    if mb.submit_messages(8).is_ok() {
        let model: u32 = mb.pop_values();
        let revision: u32 = mb.pop_values();
        let serial: u64 = mb.pop_values();
        Some(BoardInfo{model, revision, serial })
    } else {
        None
    }
}

pub fn get_mac_address() -> Option<[u8;6]>{
    use mailbox::PropertyMessageRequest::*;
    let mut mb = mailbox::Mailbox::<256>::new();
    mb.push_tag(HwGetBoardMacAddress);
    mb.push_tag(Null);
    if mb.submit_messages(8).is_ok() {
        let board_mac_address: [u8;6] = mb.pop_values();
        Some(board_mac_address)
    } else {
        None
    }
}

