pub struct BufferedWriter {
    buf: crate::collections::sync_ring::AtomicRing256<u8>,
}