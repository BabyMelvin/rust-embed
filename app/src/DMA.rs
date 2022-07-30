/// A singleton that represents a single DMA channel (channel 1 in this case)
///
/// This singleton has exclusive access to the registers of the DMA channel 1
pub struct Dma1Channel1 {
    // ..
}

impl Dma1Channel1 {

    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_destination_address(&mut self, address: usize, inc: bool) {

    }

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_source_address(&mut self, address: usize, inc: bool) {

    }

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_transfer_length(&mut self, len: usize) {

    }

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    pub fn start(&mut self) {
        // ..
    }

    /// Stops the DMA transfer
    ///
    /// NOTE this performs a volatile write
    pub fn stop(&mut self) {
        // ..
    }

    /// Returns `true` if there's a transfer in progress
    ///
    /// NOTE this performs a volatile read
    pub fn in_process() -> bool {

    }
}

/// A singleton that represents serial port #1
pub struct Serial1 {
    // NOTE: we extend this struct by adding the DMA channel singleton
    dma: Dma1Channel1,
}

impl Serial1 {
    /// Reads out a single byte
    ///
    /// NOTE: blocks if no byte is available to be read
    pub fn read(&mut self) -> Result<u8, Error> {

    }

    /// Sends out a single byte
    ///
    /// NOTE: blocks if the output FIFO buffer is full
    pub fn write(&mut self, byte: u8) -> Result<(), Error> {
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all<B> (mut self, buffer: Pin<B>) -> Transfer<B>
    where
        B: Deref + 'static,
        B::Target: AsSlice<Element = u8>,
    {
        let slice = buffer.as_slice();
        let (ptr, len) = (slice.as_ptr(), slice.len());

        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);

        self.dma.start();
        Transfer {
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }

    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact<B>(&mut self, buffer: Pin<B>) -> Transfer<B>
    where
        B: DerefMut + 'static,
        B::Target: AsSlice<Element = u8> + Unpin,
    {
        let slice = buffer.as_mut_slice();
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());

        self.dma.set_source_address(USART1_TX, false);
        self.dma.set_destination_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);

        self.dma.start();

        Transfer {
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }
}

struct Inner<B> {
    buffer: Pin<B>,
    serial: Serial1,
}

pub struct Transfer<B> {
    inner: Option<Inner<B>>,
}

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_process()
    }

    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> (B, Serial1) {
        // Busy wait until the transfer is done
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        let inner = self.inner
                    .take()
                    .unwrap_or_else(||unsafe{hint::unreachable_unchecked()});

        (self.buffer, serial.serial)
    }
}

impl<B> Drop for Transfer<B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            // NOTE: this is a volatile write
            inner.serial.dma.stop();

             // we need a read here to make the Acquire fence effective
            // we do *not* need this if `dma.stop` does a RMW operation
            unsafe {
                ptr::read_volatile(&0);
            }

            // we need a fence here for the same reason we need one in `Transfer.wait`
            atomic::compiler_fence(Ordering::Acquire);
        }
    }
}

fn write(serial: Serial1) {
    // fire and forget
    serial.write_all(b"Hello, world\n");

    // do other stuff
}

fn read(mut serial: Serial1, buf: &'static mut [u8; 16]) {
    let mut buf = [0; 16];

    let t = serial.read_exact(&mut buf);

    // do other stuff

    let (serial, buf) = t.wait();

    match buf.split(|b| * b == b'\n').next() {
        Some(b"some-command") => {}
        _=> {}
    }
}

fn reorder(serial: Serial1, buf: &'static mut [u8])  {
    // zero the buffer (for no particular reason)
    buf.iter_mut().for_each(|byte| *byte = 0);

    let t = serial.read_exact(buf);

    // ... do other stuff ..

    let (buf, serial) = t.wait();

    // 编译器可能将这条移到t.wait之前
    buf.reverse();
    // .. do stuff with `buf` ..
}
