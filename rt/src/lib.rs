#![no_std]

// #![feature(lang_items)]

// #[lang = "eh_personality"]
// extern "C" fn eh_personality()
// {
// }
use core::{panic::PanicInfo, ptr, arch::global_asm};

global_asm!(include_str!("asm.s"));

pub union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

extern "C" {
    fn NMI();
    // fn HardFault();
    fn HardFaultTrampoline();
    fn MemManage();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn PendSV();
    fn SysTick();
}

#[link_section = ".vector_table.exceptions"]
#[no_mangle]
pub static EXCEPTIONS: [Vector; 14] = [
    Vector { handler: NMI },
    // Vector { handler: HardFault },
    Vector { handler: HardFaultTrampoline },
    Vector { handler: MemManage },
    Vector { handler: BusFault },
    Vector {
        handler: UsageFault,
    },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: SVCall },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: PendSV },
    Vector { handler: SysTick },
];

#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {

    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    // Initialize RAM
    extern "C" {
        static mut _sbss: u8;
        static mut _ebss: u8;

        static mut _sdata: u8;
        static mut _edata: u8;
        static _sidata: u8;
    }

    let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
    ptr::write_bytes(&mut _sbss as *mut u8, 0, count);

    let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
    ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);

    // Call user entry point
    extern "Rust" {
        fn main() -> !;
    }
    main()
}

#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static REST_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[macro_export]
macro_rules! entry {
    ($path: path) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            let f: fn() -> ! = $path;
            f()
        }
    };
}
