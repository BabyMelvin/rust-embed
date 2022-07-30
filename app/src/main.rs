#![no_std]
#![no_main]

use cortex_m::interrupt;
use cortex_m_semihosting::{hio::{self, HStdout}, debug};

use log::{Log, log, warn, error, global_logger, GlobalLog};
use rt::entry;
use core::fmt::Write as _;

entry!(main);

static RODATA: &[u8] = b"Hello, world!";
static mut BSS: u8 = 0;
static mut DATA: u16 = 1;

struct Logger {
    hstdout: HStdout,
}

struct SLogger;

impl Log for Logger {
    type Error = ();

    fn log(&mut self, address: u8) -> Result<(), Self::Error> {
        self.hstdout.write_all(&[address])
    }
}

impl GlobalLog for SLogger {
    fn log(&self, address: u8) {
        // we use a critical section (`interrupt::free`) to make the access to the
        // `static mut` variable interrupt safe which is required for memory safety
        interrupt::free(|_| unsafe {
            static mut HSTDOUT: Option<HStdout> = None;

            // lazy initialization
            if HSTDOUT.is_none() {
                HSTDOUT = Some(hio::hstdout()?);
            }

            let hstdout = HSTDOUT.as_mut().unwrap();

            hstdout.write_all(&[address])
        }).ok(); // `.ok()` = ignore errors
    }
}

global_logger!(SLogger);

pub fn main() -> ! {

    // 符号表看到的是: 你好
    #[export_name = "你好"]
    #[link_section = ".log"]
    #[used]
    static A: u8 = 0;

    // 打印符号的地址
    let mut hstout = hio::hstdout().unwrap();
    // let _ = writeln!(hstout, "{:#x}", &A as *const u8 as usize);
    let address = &A as *const u8 as usize as u8;
    hstout.write_all(&[address]).unwrap();


    #[export_name="Goodbye"]
    #[link_section = ".log"]
    static B: u8 = 0;
    let address = &B as *const u8 as usize as u8;
    // let _ = writeln!(hstout, "{:#x}", &B as *const u8 as usize);
    hstout.write_all(&[address]).unwrap();


    let _x = RODATA;
    let _y = unsafe {
        &BSS
    };

    let _y = unsafe {
        &DATA
    };


    // 使用Logger
    let hstdout = hio::hstdout().unwrap();
    let mut logger = Logger{hstdout};
    let _ = log!(logger, "Hello world!");
    let _ = log!(logger, "Goodbye!");

    let _ = warn!(logger, "Hello, world!"); // <- CHANGED!

    let _ = error!(logger, "Goodbye"); // <- CHANGED!

    debug::exit(debug::EXIT_SUCCESS);

    loop {

    }
}

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    // do something interesting here
    loop {}
}
