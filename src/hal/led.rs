use crate::{peripherals::mailbox, system};

pub fn status_set(on: bool) {
    use mailbox::Mailbox;
    use mailbox::PropertyMessageRequest::*;
    let mut mailbox = Mailbox::<32>::new();
    let status = if on {
        mailbox::LedStatus::On
    } else {
        mailbox::LedStatus::Off
    };
    mailbox.push_tag(SetOnboardLedStatus {
        pin_number: mailbox::Led::Status,
        status,
    });
    mailbox.push_tag(Null);
    mailbox.submit_messages(8).unwrap();
}

pub fn status_get() -> bool {
    use mailbox::Mailbox;
    use mailbox::PropertyMessageRequest::*;
    let mut mailbox = Mailbox::<32>::new();
    mailbox.push_tag(GetOnboardLedStatus);
    mailbox.push_tag(Null);
    if mailbox.submit_messages(8).is_ok() {
        let (_, status): (mailbox::Led, mailbox::LedStatus) = mailbox.pop_values();
        match status {
            mailbox::LedStatus::On => true,
            _ => false
        }
    } else {
        false
    }
}

pub fn status_blink_twice(interval_msec: usize) {
    let status = status_get();
    status_set(!status);
    system::wait_msec(interval_msec);
    status_set(status);
    system::wait_msec(interval_msec);
    status_set(!status);
    system::wait_msec(interval_msec);
    status_set(status);
}
