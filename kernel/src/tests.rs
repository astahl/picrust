use crate::system::hal::clocks;
use crate::system::peripherals;
use crate::system::peripherals::dma::DmaControlAndStatus;
use crate::system::peripherals::dma::DmaControlBlock;
use crate::system::peripherals::dma::DmaTransferInformation;
use crate::system::peripherals::dma::DMA_0;
use crate::system::peripherals::uart::UART_0;
use crate::system::peripherals::usb::DwHciCoreAhbCfg;
use crate::system::peripherals::usb::DwHciCoreInterrupts;

use super::system;
use super::hal;
use mystd::collections;
use mystd::io::Write;

pub fn run() {
    use core::fmt::Write;
    let mut uart = UART_0;

    writeln!(&mut uart, "{:#?}", clocks::ClockDescription::get(clocks::Clock::ARM).unwrap());
    writeln!(&mut uart, "Current Exception Level: {}", system::current_exception_level());
    // Uart0::puts("start");

    let mut str_buffer = collections::ring::RingArray::<u8, 1024>::new();

    use hal::framebuffer::color;
    let resolution = hal::display::Resolution::preferred().unwrap_or_default();

    let fb = hal::framebuffer::Framebuffer::new(
        resolution.horizontal as u32,
        resolution.vertical as u32,
    )
    .unwrap();

    fb.clear(color::BLACK);

    let font = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(super::__font_start),
            core::ptr::addr_of!(super::__font_end)
                .offset_from(core::ptr::addr_of!(super::__font_start))
                .unsigned_abs(),
        )
    };

    let text = b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
    let mapping = |c: u8| -> u8 {
        match c {
            0 => b' ',
            b' '..=b'?' => c,
            b'@'..=b'^' => c as u8 - b'@',
            b'a'..=b'z' => c as u8 - b'`' | 0x80,
            b'{' => b'<',
            b'}' => b'>',
            b'\n' => b' ', // TODO better handle newlines in the buffer writer
            b'_' => 82,
            _ => 255,
        }
    };
    fb.clear(color::BLUE);
    fb.write_text(text, font, mapping);

    hal::led::status_blink_twice(500);
    fb.clear(color::RED);

    let mut supported_resolutions = [hal::display::Resolution::default(); 128];
    let count = hal::display::Resolution::supported(supported_resolutions.as_mut_slice(), 0);
    writeln!(
        str_buffer,
        "Supported {:?}",
        supported_resolutions.get(0..count)
    )
    .unwrap();
    writeln!(str_buffer, "Requested Resolution {:?}", resolution).unwrap();
    writeln!(
        str_buffer,
        "Framebuffer: {} {} {}",
        fb.width_px, fb.height_px, fb.bits_per_pixel
    )
    .unwrap();
    if let Some(arm_memory) = hal::info::get_arm_memory() {
        writeln!(str_buffer, "ARM {}", arm_memory).unwrap();
    }
    if let Some(vc_memory) = hal::info::get_vc_memory() {
        writeln!(str_buffer, "VC {}", vc_memory).unwrap();
    }
    // if let Some(board_info) = hal::info::get_board_info() {
    //     writeln!(str_buffer, "{}", board_info.revision).unwrap();
    // }
    // if let Some(mac) = hal::info::get_mac_address() {
    //     writeln!(str_buffer, "MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).unwrap();
    // }

    for edid in hal::display::EdidIterator::new() {
        writeln!(str_buffer, "EDID BLOCK {:?}", edid).unwrap();
        // for byte in edid.bytes() {
        //     write!(str_buffer, "{:02X} ", byte).unwrap();
        // }
    }
    writeln!(str_buffer, "Bye!").unwrap();
    let text = str_buffer.to_str().unwrap();
    fb.clear(color::BLACK);
    fb.write_text(text.as_bytes(), font, mapping);

    uart.write_all(text.as_bytes());
    // Uart0::put_uint(core as u64);
    // Uart0::puts("Hallo\n");
    //

    // fb.set_pixel_a8b8g8r8(150, 100, color::WHITE);
    // let mut canvas = drawing::PixelCanvas::with_slice(
    //     fb.width_px as usize,
    //     fb.height_px as usize,
    //     fb.pitch_bytes as usize / 4,
    //     fb.as_mut_pixels(),
    // )
    // .unwrap();
    // //canvas.clear(color::BLUE);
    // canvas
    //     .fill_rect(color::BLUE, (298, 298), (300, 300))
    //     .unwrap();
    // canvas.fill_lines(color::RED, 100..=100).unwrap();
    // let pixelscale = (2, 2);
    // let cols = canvas.width / (pixelscale.0 * 8);
    // let rows = canvas.height / (pixelscale.1 * 8);
    // let mut row_buffer = [0_u64; 256];
    // let mut v_scroll: usize = 0;
    // hal::led::status_set(false);
    // loop {
    //     let line_iterator = text
    //         .split(|b| *b == b'\n')
    //         .flat_map(|l| l.chunks(cols))
    //         .cycle();
    //     canvas.fill_rect(0, (0, 0), (cols * 8, rows * 8)).unwrap();
    //     for (row_nr, text_line) in line_iterator.skip(v_scroll as usize).take(rows).enumerate() {
    //         let mut pre = 0;
    //         let mut len = 0;
    //         for (dst, src) in row_buffer.iter_mut().zip(text_line) {
    //             let val = font[mapping(*src) as usize];
    //             if len == 0 && val == 0 {
    //                 pre += 1;
    //                 continue;
    //             }
    //             *dst = val;
    //             len += 1;
    //         }
    //         canvas
    //             .blit8x8_line(
    //                 &row_buffer[pre..len + pre],
    //                 color::WHITE,
    //                 color::BLACK,
    //                 (pre * 8, row_nr * 8),
    //             )
    //             .unwrap();
    //     }
    //     canvas.scale_in_place(pixelscale.0, pixelscale.1);
    //     v_scroll += 1;

    //     system::wait_msec(100);
    // }
}

pub fn test_dma() {
    use core::fmt::Write;
    let mut uart = UART_0;
    use super::peripherals::dma;
    let mut str_buffer = collections::ring::RingArray::<u8, 1024>::new();
    // let mut status = dma::Dma0::control_status();
    // status.set_reset();
    // writeln!(str_buffer, "{:?}", dma::Dma0::control_status()).unwrap();
    // Dma0::set_control_status(status);
    // while Dma0::control_status().is_resetting() {
    //     writeln!(str_buffer, "RESETTING {:?}", dma::Dma0::control_status()).unwrap();
    // }

    let mem_start = unsafe { core::ptr::addr_of_mut!(super::__kernel_end).wrapping_add(0x100000) };

    writeln!(str_buffer, "MEM START = {:x}", mem_start as usize).unwrap();
    let control_block_ptr: *mut dma::DmaControlBlock = mem_start.cast();
    let src = mem_start.wrapping_add(0x100000).cast::<u8>();
    let dest = src.wrapping_add(0x100000);
    writeln!(str_buffer, "CB Addr = {:p}", control_block_ptr).unwrap();
    writeln!(str_buffer, "Src Addr = {:p}", src).unwrap();
    writeln!(str_buffer, "Dest Addr = {:p}", dest).unwrap();
    uart.write_all(str_buffer.make_continuous());
    str_buffer.clear();
    
    unsafe {
        let length = 8*1024*1024;
        core::slice::from_raw_parts_mut(src, length).fill(0x55);
        let transfer_information = DmaTransferInformation::wide_copy();
        let cb = DmaControlBlock::linear_copy(transfer_information, src as u32, dest as u32, length as u32, 0);
        control_block_ptr.write_volatile(cb);
        
        DMA_0.set_control_block_address(control_block_ptr as u32);

        writeln!(str_buffer, "Src = {:x}", src.read()).unwrap();
        writeln!(str_buffer, "Dest = {:x}", dest.read()).unwrap();
        writeln!(str_buffer, "cb: {:x}", DMA_0.control_block_address()).unwrap();
        let status = DMA_0.control_and_status()
            .with_active_set()
            .with_axi_priority_level(DmaControlAndStatus::MAX_PRIORITY_LEVEL).unwrap()
            .with_axi_panic_priority_level(DmaControlAndStatus::MAX_PRIORITY_LEVEL).unwrap()
            .with_wait_for_outstanding_writes_set();
        DMA_0.set_control_and_status(status);
        while !DMA_0.control_and_status().is_end() {
            writeln!(str_buffer, "wait").unwrap();
        }
        writeln!(str_buffer, "cb: {:x}", DMA_0.control_block_address()).unwrap();
        writeln!(str_buffer, "Dest = {:x}", dest.read()).unwrap();
        writeln!(str_buffer, "Ended? {:?}", DMA_0.control_and_status().is_end()).unwrap();

        writeln!(str_buffer, "dbg: {:#?}", DMA_0.debug()).unwrap();
    }
    uart.write_all(str_buffer.make_continuous());
    

}

pub fn test_usb() -> Option<()>{
    use core::time::Duration;
    use core::fmt::Write;
    use peripherals::usb;
    use peripherals::power;
    let mut uart = UART_0;

    writeln!(&mut uart, "USB Vendor-ID {:#x}", usb::DwHciCore::vendor_id()).ok()?;

    let power_state = power::PowerDevice::USBHCD.state()?;
    writeln!(&mut uart, "USB Exists: {}", power_state.exists()).ok()?;
    writeln!(&mut uart, "USB Power On: {}", power_state.is_on()).ok()?;
    if !power_state.is_on() {
        let timeout = core::time::Duration::from_millis(power::PowerDevice::USBHCD.timing_ms()? as u64);
        writeln!(&mut uart, "USB Power On Timeout: {} msec", timeout.as_millis()).ok()?;
        let turned_on = power_state.with_on().with_wait_set();
        power::PowerDevice::USBHCD.set_state(turned_on);
        system::wait(timeout);
        let power_state = power::PowerDevice::USBHCD.state()?;
        writeln!(&mut uart, "USB Power On: {}", power_state.is_on()).ok()?;
    }

    let ahb_config = usb::DwHciCore::ahb_config()
        .with_global_interrupt_disabled();
    usb::DwHciCore::set_ahb_config(ahb_config);

    // todo! hook up irq 9 

    // DWHCIDeviceInitCore enter
    let usb_config = usb::DwHciCore::usb_config()
        .with_ulpi_ext_vbus_drv_cleared()
        .with_term_sel_dl_pulse_cleared();
    usb::DwHciCore::set_usb_config(usb_config);

    // reset dwhci device
    let mut reset = usb::DwHciCoreReset::clear();
    usb::DwHciCore::set_reset(reset);
    
	// wait for AHB master IDLE state
    reset = poll_await(usb::DwHciCore::get_reset, usb::DwHciCoreReset::is_ahb_idle_set, 100, Duration::from_millis(1)).expect("ahb should turn idle");

    // soft reset
    usb::DwHciCore::set_reset(reset.with_soft_reset_set());
    let _ = poll_await(usb::DwHciCore::get_reset, |r| !r.is_soft_reset_set(), 100, Duration::from_millis(1)).expect("soft reset bit should clear");

    system::wait(Duration::from_millis(100));
    // reset finished

    let usb_config = usb::DwHciCore::usb_config()
        .with_ulpi_utmi_sel_cleared()
        .with_phyif_cleared();
    usb::DwHciCore::set_usb_config(usb_config);

    // Internal DMA mode only
    let (_, hw_cfg2, _, _) = usb::DwHciCore::hw_config();
    let mut usb_config = usb::DwHciCore::usb_config();
    usb_config = if let (usb::FsPhyType::Dedicated, usb::HsPhyType::Ulpi) = (hw_cfg2.fs_phy_type(), hw_cfg2.hs_phy_type()) {
        usb_config.with_ulpi_clk_sus_m_set().with_ulpi_fsls_set()
    } else {
        usb_config.with_ulpi_clk_sus_m_cleared().with_ulpi_fsls_cleared()
    };
    usb::DwHciCore::set_usb_config(usb_config);

    let num_host_channels = hw_cfg2.num_host_channels();
    assert!(num_host_channels >= 4 && num_host_channels <= 16);

    let ahb_config = usb::DwHciCore::ahb_config()
        .with_dma_enabled()
        .with_wait_axi_writes_set()
        .with_max_axi_burst(0);
    usb::DwHciCore::set_ahb_config(ahb_config);

	// HNP and SRP are not used
    let usb_config = usb::DwHciCore::usb_config()
        .with_srp_capable_cleared()
        .with_hnp_capable_cleared();
    usb::DwHciCore::set_usb_config(usb_config);

    // DWHCIDeviceEnableCommonInterrupts
    // Clear any pending interrupts
    usb::DwHciCore::set_interrupt_state(DwHciCoreInterrupts::all_set());
    
    // DWHCIDeviceInitCore finished

    // DWHCIDeviceEnableGlobalInterrupts enter
    let ahb_config = usb::DwHciCore::ahb_config()
        .with_global_interrupt_enabled();
    usb::DwHciCore::set_ahb_config(ahb_config);
    // DWHCIDeviceEnableGlobalInterrupts finished

    // DWHCIDeviceInitHost enter
    // DWHCIDeviceInitHost leave

    Some(())

}

#[derive(Debug)]
struct TimeoutError();

fn poll_await<R: Copy, G: Fn() -> R, F: Fn(R) -> bool>(generate: G, predicate: F, mut timeout_count: usize, timeout_interval: core::time::Duration) -> Result<R,TimeoutError> {
    loop {
        let result = generate();
        if predicate(result) {
            break Ok(result);
        }
        if timeout_count == 0 {
            break Err(TimeoutError());
        }
        timeout_count -= 1;
        system::wait(timeout_interval);
    }
}