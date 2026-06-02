use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// PL011 UART0 base address (QEMU raspi4b / BCM2711)
#[cfg(target_board = "raspi4b")]
const PL011_BASE: usize = 0xFE20_1000;
#[cfg(target_board = "versatilepb")]
const PL011_BASE: usize = 0x101F_1000;

register_bitfields! [
    u32,

    /// Data Register
    DR [
        DATA OFFSET(0) NUMBITS(8) []
    ],

    /// Flag Register
    FR [
        /// Transmit FIFO full
        TXFF OFFSET(5) NUMBITS(1) [],
        /// Receive FIFO empty
        RXFE OFFSET(4) NUMBITS(1) [],
        /// UART busy
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// Line Control Register
    LCR_H [
        /// Word length
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBits  = 0b00,
            SixBits   = 0b01,
            SevenBits = 0b10,
            EightBits = 0b11
        ],
        /// Enable FIFOs
        FEN OFFSET(4) NUMBITS(1) []
    ],

    /// Control Register
    CR [
        /// Receive enable
        RXE OFFSET(9) NUMBITS(1) [],
        /// Transmit enable
        TXE OFFSET(8) NUMBITS(1) [],
        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) []
    ],

    /// Interrupt Clear Register
    ICR [
        ALL OFFSET(0) NUMBITS(11) []
    ]
];

register_structs! {
    #[allow(non_snake_case)]
    pub Pl011Regs {
        (0x000 => DR: ReadWrite<u32, DR::Register>),
        (0x004 => _rsr_ecr),
        (0x018 => FR: ReadOnly<u32, FR::Register>),
        (0x01C => _reserved0),
        (0x024 => IBRD: ReadWrite<u32>),
        (0x028 => FBRD: ReadWrite<u32>),
        (0x02C => LCR_H: ReadWrite<u32, LCR_H::Register>),
        (0x030 => CR: ReadWrite<u32, CR::Register>),
        (0x034 => _ifls),
        (0x038 => IMSC: ReadWrite<u32>),
        (0x03C => _ris),
        (0x040 => _mis),
        (0x044 => ICR: WriteOnly<u32, ICR::Register>),
        (0x048 => @END),
    }
}

pub struct Uart {
    regs: &'static Pl011Regs,
}

impl Uart {
    /// # Safety
    /// Caller must ensure `PL011_BASE` is the correct MMIO address for the UART.
    pub unsafe fn new() -> Self {
        Self {
            regs: unsafe { &*(PL011_BASE as *const Pl011Regs) },
        }
    }

    pub fn init(&self) {
        // Disable UART
        self.regs.CR.set(0);

        // Clear pending interrupts
        self.regs.ICR.write(ICR::ALL::SET);

        // 115200 baud @ 48 MHz (QEMU raspi4b ref clock): IBRD=26, FBRD=3
        self.regs.IBRD.set(26);
        self.regs.FBRD.set(3);

        // 8-bit, enable FIFO
        self.regs
            .LCR_H
            .write(LCR_H::WLEN::EightBits + LCR_H::FEN::SET);

        // Mask all interrupts
        self.regs.IMSC.set(0);

        // Enable UART, TX, RX
        self.regs
            .CR
            .write(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }

    /// Blocking write: wait until TX FIFO not full, then send one byte.
    pub fn putc(&self, c: u8) {
        while self.regs.FR.is_set(FR::TXFF) {}
        self.regs.DR.write(DR::DATA.val(c as u32));
    }

    /// Blocking read: wait until RX FIFO not empty, then return one byte.
    pub fn getc(&self) -> u8 {
        while self.regs.FR.is_set(FR::RXFE) {}
        self.regs.DR.read(DR::DATA) as u8
    }

    /// Write a string byte by byte.
    pub fn puts(&self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.putc(b'\r');
            }
            self.putc(b);
        }
    }
}
