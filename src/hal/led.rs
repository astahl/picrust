use crate::peripherals::mailbox;
use crate::delay;

pub fn status_set(on: bool) {
    use mailbox::Mailbox;
    use mailbox::PropertyMessageRequest::*;
    let mut mailbox = Mailbox::<256>::new();
    let status = if on { mailbox::LedStatus::On } else { mailbox::LedStatus::Off};
    let _ = mailbox.request(8, &[
        SetOnboardLedStatus { pin_number: mailbox::Led::Status, status },
        Null]);
}

pub fn status_blink() {
    status_set(true);
    delay(100000);
    status_set(false);
    delay(100000);
}