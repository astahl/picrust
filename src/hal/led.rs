use crate::peripherals::mailbox;
use crate::delay;

pub fn led_on() {
    use mailbox::Mailbox;
    use mailbox::PropertyMessageRequest::*;
    let mut mailbox = Mailbox::<256>::new();
    let _ = mailbox.request(8, &[
        SetOnboardLedStatus { pin_number: mailbox::Led::Status, status: mailbox::LedStatus::On },
        Null]);
}

pub fn led_off() {
    use mailbox::Mailbox;
    use mailbox::PropertyMessageRequest::*;
    let mut mailbox = Mailbox::<256>::new();
    let _ = mailbox.request(8, &[
        SetOnboardLedStatus { pin_number: mailbox::Led::Status, status: mailbox::LedStatus::Off },
        Null]);
}

pub fn blink_led() {
    led_on();
    delay(100000);
    led_off();
    delay(100000);
}