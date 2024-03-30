use core::slice;

use crate::println_debug;
use crate::println_log;
use crate::system::hal::clocks;
use crate::system::hal::framebuffer::Framebuffer;
use crate::system::peripherals;
use crate::system::peripherals::dma::DmaControlAndStatus;
use crate::system::peripherals::dma::DmaControlBlock;
use crate::system::peripherals::dma::DMA_0;
use crate::system::peripherals::uart::UART_0;
use crate::system::peripherals::usb::DwHciCoreInterrupts;

use super::hal;
use super::system;
use mystd::arr2d;
use mystd::byte_value::ByteValue;
use mystd::collections::rectangular::RectangularArray;
use mystd::io::Write;
use mystd::slice::slice2d::traits::MutSlice2dTrait;
use mystd::slice::slice2d::traits::Slice2dTrait;

// pub fn run() {
//     println_log!("{:#?}", clocks::ClockDescription::get(clocks::Clock::ARM));
//     println_log!(
//         "Current Exception Level: {}",
//         system::arm_core::current_exception_level()
//     );

//     use core::fmt::Write;
//     let mut str_buffer = collections::ring::RingArray::<u8, 1024>::new();

//     use hal::framebuffer::color;
//     let resolution = hal::display::Resolution::preferred().unwrap_or_default();

//     let fb = hal::framebuffer::Framebuffer::new(
//         resolution.horizontal as u32,
//         resolution.vertical as u32,
//         32,
//     )
//     .unwrap();

//     fb.clear(color::BLACK);

//     let font: &'static [u64] = unsafe {
//         core::slice::from_raw_parts(include_bytes!("../901447-10.bin").as_ptr().cast(), 256)
//     };

//     let text = b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
//     let mapping = |c: u8| -> u8 {
//         match c {
//             0 => b' ',
//             b' '..=b'?' => c,
//             b'@'..=b'^' => c as u8 - b'@',
//             b'a'..=b'z' => c as u8 - b'`' | 0x80,
//             b'{' => b'<',
//             b'}' => b'>',
//             b'\n' => b' ', // TODO better handle newlines in the buffer writer
//             b'_' => 82,
//             _ => 255,
//         }
//     };
//     fb.clear(color::BLUE);
//     fb.write_text(text, font, mapping);

//     hal::led::status_blink_twice(500);
//     fb.clear(color::RED);

//     let mut supported_resolutions = [hal::display::Resolution::default(); 128];
//     let count = hal::display::Resolution::supported(supported_resolutions.as_mut_slice(), 0);
//     writeln!(
//         str_buffer,
//         "Supported {:?}",
//         supported_resolutions.get(0..count)
//     )
//     .unwrap();
//     writeln!(str_buffer, "Requested Resolution {:?}", resolution).unwrap();
//     writeln!(
//         str_buffer,
//         "Framebuffer: {} {} {}",
//         fb.width_px, fb.height_px, fb.bits_per_pixel
//     )
//     .unwrap();
//     if let Some(arm_memory) = hal::info::get_arm_memory() {
//         writeln!(str_buffer, "ARM {}", arm_memory).unwrap();
//     }
//     if let Some(vc_memory) = hal::info::get_vc_memory() {
//         writeln!(str_buffer, "VC {}", vc_memory).unwrap();
//     }
//     // if let Some(board_info) = hal::info::get_board_info() {
//     //     writeln!(str_buffer, "{}", board_info.revision).unwrap();
//     // }
//     // if let Some(mac) = hal::info::get_mac_address() {
//     //     writeln!(str_buffer, "MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).unwrap();
//     // }

//     for edid in hal::display::EdidIterator::new() {
//         writeln!(str_buffer, "EDID BLOCK {:?}", edid).unwrap();
//         // for byte in edid.bytes() {
//         //     write!(str_buffer, "{:02X} ", byte).unwrap();
//         // }
//     }
//     writeln!(str_buffer, "Bye!").unwrap();
//     let text = str_buffer.to_str().unwrap();
//     fb.clear(color::BLACK);
//     fb.write_text(text.as_bytes(), font, mapping);

//     println_log!("{text}");
//     // Uart0::put_uint(core as u64);
//     // Uart0::puts("Hallo\n");
//     //

//     // fb.set_pixel_a8b8g8r8(150, 100, color::WHITE);
//     // let mut canvas = drawing::PixelCanvas::with_slice(
//     //     fb.width_px as usize,
//     //     fb.height_px as usize,
//     //     fb.pitch_bytes as usize / 4,
//     //     fb.as_mut_pixels(),
//     // )
//     // .unwrap();
//     // //canvas.clear(color::BLUE);
//     // canvas
//     //     .fill_rect(color::BLUE, (298, 298), (300, 300))
//     //     .unwrap();
//     // canvas.fill_lines(color::RED, 100..=100).unwrap();
//     // let pixelscale = (2, 2);
//     // let cols = canvas.width / (pixelscale.0 * 8);
//     // let rows = canvas.height / (pixelscale.1 * 8);
//     // let mut row_buffer = [0_u64; 256];
//     // let mut v_scroll: usize = 0;
//     // hal::led::status_set(false);
//     // loop {
//     //     let line_iterator = text
//     //         .split(|b| *b == b'\n')
//     //         .flat_map(|l| l.chunks(cols))
//     //         .cycle();
//     //     canvas.fill_rect(0, (0, 0), (cols * 8, rows * 8)).unwrap();
//     //     for (row_nr, text_line) in line_iterator.skip(v_scroll as usize).take(rows).enumerate() {
//     //         let mut pre = 0;
//     //         let mut len = 0;
//     //         for (dst, src) in row_buffer.iter_mut().zip(text_line) {
//     //             let val = font[mapping(*src) as usize];
//     //             if len == 0 && val == 0 {
//     //                 pre += 1;
//     //                 continue;
//     //             }
//     //             *dst = val;
//     //             len += 1;
//     //         }
//     //         canvas
//     //             .blit8x8_line(
//     //                 &row_buffer[pre..len + pre],
//     //                 color::WHITE,
//     //                 color::BLACK,
//     //                 (pre * 8, row_nr * 8),
//     //             )
//     //             .unwrap();
//     //     }
//     //     canvas.scale_in_place(pixelscale.0, pixelscale.1);
//     //     v_scroll += 1;

//     //     system::wait_msec(100);
//     // }
// }

pub fn test_screen() {
    use super::system::screen::*;
    let ptr = ByteValue::from_mibi(128).as_bytes() as *mut u8;
    let slice = unsafe {
        slice::from_raw_parts_mut(ptr, ByteValue::from_mibi(16).as_bytes() as usize)
    };
    let geom = ScreenGeometry::with_size(Size { width: 640, height: 480 });
    let mut screen: Screen<u8> = Screen::try_create_in_slice(slice, geom).expect("Creating the screen should work");
    Palette::vga().make_current();
    
    for i in 0..256 {
        for col in 0..=screen.width() {
            
            screen.draw(|slice2d| {
                let (mut left, mut right) = slice2d.split_at_col_mut(col);
                left.fill(((i + 1) % 256) as u8);
                right.fill(i as u8)
            });
            
            screen.present(SwapStrategy::Swap, PresentStrategy::Memcopy);
        }
    }
}

pub fn test_irq() {
    use crate::peripherals::interrupts;
    interrupts::BasicIrqs::all_set().write_disable();
    interrupts::GpuIrqs1::all_set().write_disable();
    interrupts::GpuIrqs2::zero().uart_int().set().write_disable();
    interrupts::irq_enable();
}

pub fn test_dma() {
    use super::peripherals::dma;

    let src = [0x0f_u8;1024];
    let mut dst = [0x00_u8;1024];
    dma::dma_copy_slice(&src, &mut dst);
    assert_eq!(src, dst);

    let src_buf = arr2d!([1_u32,1,1,1,0,0], [1,1,1,1,0,0], [1,1,1,1,0,0], [1,1,1,1,0,0], [0,0,0,0,0,0]);
    let mut dst_buf: RectangularArray<u32, 8, 8> = RectangularArray::new();
    let mut dst = dst_buf.sub_mut_slice2d((1..5, 1..5));
    let src = src_buf.sub_slice2d((..4, ..4));
    dma::dma_copy_slice2d(&src, &mut dst);
    assert_eq!(src, dst);

    // let mut status = dma::Dma0::control_status();
    // status.set_reset();
    // writeln!(str_buffer, "{:?}", dma::Dma0::control_status()).unwrap();
    // Dma0::set_control_status(status);
    // while Dma0::control_status().is_resetting() {
    //     writeln!(str_buffer, "RESETTING {:?}", dma::Dma0::control_status()).unwrap();
    // }

    let mem_start = unsafe { (1024 * 1024 * 16) as *mut u8 };

    println_log!("MEM START = {:x}", mem_start as usize);
    let control_block_ptr: *mut dma::DmaControlBlock = mem_start.cast();
    let src = mem_start.wrapping_add(0x100000).cast::<u8>();
    let dest = src.wrapping_add(0x100000);
    println_log!("CB Addr = {:p}", control_block_ptr);
    println_log!("Src Addr = {:p}", src);
    println_log!("Dest Addr = {:p}", dest);

    unsafe {
        let length = 8 * 1024 * 1024;
        core::slice::from_raw_parts_mut(src, length).fill(0x55);
        let transfer_information = dma::DmaTransferInformation::wide_copy();
        let cb = DmaControlBlock::new_linear_copy(
            transfer_information,
            src as u32,
            dest as u32,
            length as u32,
            0,
        );
        println_log!("cb = {:#?}", &cb);
        control_block_ptr.write_volatile(cb);

        DMA_0.control_block_address().write(control_block_ptr as u32);

        println_log!("Src = {:x}", src.read());
        println_log!("Dest = {:x}", dest.read());
        println_log!("cb: {:x}", DMA_0.control_block_address().read());
        DMA_0
            .control_and_status().update(|cs|
                cs
            .active()
            .set()
            .axi_priority_level()
            .set_value(DmaControlAndStatus::MAX_PRIORITY_LEVEL)
            .axi_panic_priority_level()
            .set_value(DmaControlAndStatus::MAX_PRIORITY_LEVEL)
            .wait_for_outstanding_writes()
            .set());
        
        DMA_0.wait_for_end();
        println_log!("cb: {:x}", DMA_0.control_block_address().read());
        println_log!("Dest = {:x}", dest.read());
        println_log!("Ended? {:?}", DMA_0.control_and_status().read().end().is_set());
        println_log!("dbg: {:#?}", DMA_0.debug().read());
    }
}

pub fn test_usb() -> Option<()> {
    use core::time::Duration;
    use peripherals::power;
    use peripherals::usb;

    println_log!("USB Vendor-ID {:#x}", usb::DwHciCore::vendor_id());

    let power_state = power::PowerDevice::USBHCD.state()?;
    println_log!("USB Exists: {}", power_state.exists());
    println_log!("USB Power On: {}", power_state.is_on());
    if !power_state.is_on() {
        let timeout =
            core::time::Duration::from_millis(power::PowerDevice::USBHCD.timing_ms()? as u64);
        println_log!("USB Power On Timeout: {} msec", timeout.as_millis());
        let turned_on = power_state.with_on().with_wait_set();
        power::PowerDevice::USBHCD.set_state(turned_on);
        system::arm_core::counter::wait(timeout);
        let power_state = power::PowerDevice::USBHCD.state()?;
        println_log!("USB Power On: {}", power_state.is_on());
    }

    let ahb_config = usb::DwHciCore::ahb_config().enable_global_interrupt().set();
    usb::DwHciCore::set_ahb_config(ahb_config);

    // todo! hook up irq 9

    // DWHCIDeviceInitCore enter
    let usb_config = usb::DwHciCore::usb_config()
        .ulpi_ext_vbus_drv()
        .clear()
        .term_sel_dl_pulse()
        .clear();
    usb::DwHciCore::set_usb_config(usb_config);

    // reset dwhci device
    let mut reset = usb::DwHciCoreReset::zero();
    usb::DwHciCore::set_reset(reset);

    // wait for AHB master IDLE state
    reset = poll_await(
        usb::DwHciCore::get_reset,
        |r| r.ahb_idle().is_set(),
        100,
        Duration::from_millis(1),
    )
    .expect("ahb should turn idle");

    // soft reset
    usb::DwHciCore::set_reset(reset.soft_reset().set());
    let _ = poll_await(
        usb::DwHciCore::get_reset,
        |r| r.soft_reset().is_clear(),
        100,
        Duration::from_millis(1),
    )
    .expect("soft reset bit should clear");

    system::arm_core::counter::wait(Duration::from_millis(100));
    // reset finished

    let usb_config = usb::DwHciCore::usb_config()
        .ulpi_utmi_sel()
        .clear()
        .phyif()
        .clear();
    usb::DwHciCore::set_usb_config(usb_config);

    // Internal DMA mode only
    let (_, hw_cfg2, _, _) = usb::DwHciCore::hw_config();
    let mut usb_config = usb::DwHciCore::usb_config();
    usb_config = if let (Ok(usb::FsPhyType::Dedicated), Ok(usb::HsPhyType::Ulpi)) =
        (hw_cfg2.fs_phy_type().value(), hw_cfg2.hs_phy_type().value())
    {
        usb_config.ulpi_clk_sus_m().set().ulpi_fsls().set()
    } else {
        usb_config.ulpi_clk_sus_m().clear().ulpi_fsls().clear()
    };
    usb::DwHciCore::set_usb_config(usb_config);

    let num_host_channels = hw_cfg2.num_host_channels_actual();
    assert!(num_host_channels >= 4 && num_host_channels <= 16);

    let ahb_config = usb::DwHciCore::ahb_config()
        .enable_dma()
        .set()
        .wait_axi_writes()
        .set()
        .max_axi_burst()
        .set_value(0);
    usb::DwHciCore::set_ahb_config(ahb_config);

    // HNP and SRP are not used
    let usb_config = usb::DwHciCore::usb_config()
        .srp_capable()
        .clear()
        .hnp_capable()
        .clear();
    usb::DwHciCore::set_usb_config(usb_config);

    // DWHCIDeviceEnableCommonInterrupts
    // Clear any pending interrupts
    usb::DwHciCore::set_interrupt_state(DwHciCoreInterrupts::all_set());

    // DWHCIDeviceInitCore finished

    // DWHCIDeviceEnableGlobalInterrupts enter
    let ahb_config = usb::DwHciCore::ahb_config().enable_global_interrupt().set();
    usb::DwHciCore::set_ahb_config(ahb_config);
    // DWHCIDeviceEnableGlobalInterrupts finished

    // DWHCIDeviceInitHost enter
    // DWHCIDeviceInitHost leave

    Some(())
}

#[derive(Debug)]
struct TimeoutError();

fn poll_await<R: Copy, G: Fn() -> R, F: Fn(R) -> bool>(
    generate: G,
    predicate: F,
    mut timeout_count: usize,
    timeout_interval: core::time::Duration,
) -> Result<R, TimeoutError> {
    loop {
        let result = generate();
        if predicate(result) {
            break Ok(result);
        }
        if timeout_count == 0 {
            break Err(TimeoutError());
        }
        timeout_count -= 1;
        system::arm_core::counter::wait(timeout_interval);
    }
}
