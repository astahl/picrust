use crate::delay;
use crate::peripherals::mailbox;

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

pub fn status_blink() {
    status_set(true);
    delay(100000);
    status_set(false);
    delay(100000);
}
