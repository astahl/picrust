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
    let request = [
        mailbox::PropertyMessageRequest::HwGetArmMemory,
        mailbox::PropertyMessageRequest::Null
    ];
    if let Ok(response) = mb.request(8, &request) {
        match response[0] {
            mailbox::PropertyMessageResponse::HwGetArmMemory { base_address, size } => 
                Some(Memory{
                    base_address: base_address as usize,
                    size: size as usize
                })
            ,
            _ => None
        }
    } else {
        None
    }
}

pub fn get_vc_memory() -> Option<Memory>{
    let mut mb = mailbox::Mailbox::<256>::new();
    let request = [
        mailbox::PropertyMessageRequest::HwGetVcMemory,
        mailbox::PropertyMessageRequest::Null
    ];
    if let Ok(response) = mb.request(8, &request) {
        match response[0] {
            mailbox::PropertyMessageResponse::HwGetVcMemory { base_address, size } => 
                Some(Memory{
                    base_address: base_address as usize,
                    size: size as usize
                })
            ,
            _ => None
        }
    } else {
        None
    }
}

pub fn get_board_info() -> Option<BoardInfo>{
    use mailbox::PropertyMessageRequest::*;
    let mut mb = mailbox::Mailbox::<256>::new();
    let request = [
        HwGetBoardModel,
        HwGetBoardRevision,
        HwGetBoardSerial,
        Null
    ];
    if let Ok(response) = mb.request(8, &request) {
        use mailbox::PropertyMessageResponse::*;
        let mut result = BoardInfo{model: 0, revision: 0, serial: 0 };
        let mut iter = response.iter();
        result.model = match iter.next() {
            Some(HwGetBoardModel { board_model }) => *board_model,
            _ => 0,
        };
        result.revision = match iter.next() {
            Some(HwGetBoardRevision { board_revision }) => *board_revision,
            _ => 0,
        };
        result.serial = match iter.next() {
            Some(HwGetBoardSerial { board_serial }) => *board_serial,
            _ => 0,
        };
        Some(result)
    } else {
        None
    }
}

pub fn get_mac_address() -> Option<[u8;6]>{
    use mailbox::PropertyMessageRequest::*;
    let mut mb = mailbox::Mailbox::<256>::new();
    let request = [
        HwGetBoardMacAddress,
        Null
    ];
    if let Ok(response) = mb.request(8, &request) {
        use mailbox::PropertyMessageResponse::*;
        match response[0] {
            HwGetBoardMacAddress { board_mac_address } => Some(board_mac_address),
            _ => None
        }
    } else {
        None
    }
}

