use core::{fmt, mem::MaybeUninit, ptr};

use tock_registers::{
    interfaces::Writeable,
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;
const PIXELS: usize = WIDTH * HEIGHT;
const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;
const TEXT_SCALE: usize = 2;
const TEXT_CHAR_WIDTH: usize = FONT_WIDTH * TEXT_SCALE + TEXT_SCALE;
const TEXT_LINE_HEIGHT: usize = FONT_HEIGHT * TEXT_SCALE + TEXT_SCALE;

const PL110_BASE: usize = 0x1012_0000;

const TIMING0_640X480: u32 = 0x3F1F_3F9C;
const TIMING1_640X480: u32 = 0x090B_61DF;
const TIMING2_640X480: u32 = 0x067F_1800;
const TIMING3_640X480: u32 = 0x0000_0000;

static mut FRAMEBUFFER: [u16; PIXELS] = [0; PIXELS];
static mut PRINT_WRITER: MaybeUninit<TextWriter> = MaybeUninit::uninit();
static mut PRINT_WRITER_READY: bool = false;

register_bitfields! [
    u32,

    CONTROL [
        LCDEN OFFSET(0) NUMBITS(1) [],
        LCDBPP OFFSET(1) NUMBITS(3) [
            Bpp16 = 0b100
        ],
        LCDTFT OFFSET(5) NUMBITS(1) [],
        LCDPWR OFFSET(11) NUMBITS(1) [],
        LCDVCOMP OFFSET(12) NUMBITS(2) [
            VerticalBackPorch = 0b01
        ]
    ],

    INTCLR [
        ALL OFFSET(0) NUMBITS(5) []
    ]
];

register_structs! {
    #[allow(non_snake_case)]
    pub Pl110Regs {
        (0x000 => Timing0: ReadWrite<u32>),
        (0x004 => Timing1: ReadWrite<u32>),
        (0x008 => Timing2: ReadWrite<u32>),
        (0x00C => Timing3: ReadWrite<u32>),
        (0x010 => UPBASE: ReadWrite<u32>),
        (0x014 => LPBASE: ReadWrite<u32>),
        (0x018 => CONTROL: ReadWrite<u32, CONTROL::Register>),
        (0x01C => INTMASK: ReadWrite<u32>),
        (0x020 => RAWINTR: ReadOnly<u32>),
        (0x024 => MASKEDINTR: ReadOnly<u32>),
        (0x028 => INTCLR: WriteOnly<u32, INTCLR::Register>),
        (0x02C => UPCURR: ReadOnly<u32>),
        (0x030 => LPCURR: ReadOnly<u32>),
        (0x034 => @END),
    }
}

#[derive(Clone, Copy)]
pub struct Rgb565(u16);

#[allow(dead_code)]
impl Rgb565 {
    pub const BLACK: Self = Self(0x0000);
    pub const WHITE: Self = Self(0xFFFF);
    pub const RED: Self = Self(0xF800);
    pub const GREEN: Self = Self(0x07E0);
    pub const BLUE: Self = Self(0x001F);

    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u16 & 0xF8) << 8) | ((green as u16 & 0xFC) << 3) | (blue as u16 >> 3))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

pub struct TextWriter {
    lcd: Lcd,
    cursor_x: usize,
    cursor_y: usize,
    foreground: Rgb565,
    background: Rgb565,
}

#[derive(Clone, Copy)]
pub struct Lcd {
    regs: &'static Pl110Regs,
    framebuffer: *mut u16,
}

#[allow(dead_code)]
impl Lcd {
    /// # Safety
    /// Caller must ensure `PL110_BASE` is the correct MMIO address for the CLCD.
    pub unsafe fn new() -> Self {
        Self {
            regs: unsafe { &*(PL110_BASE as *const Pl110Regs) },
            framebuffer: framebuffer_base(),
        }
    }

    pub fn init(&self) {
        self.regs.CONTROL.set(0);
        self.regs.INTMASK.set(0);
        self.regs.INTCLR.write(INTCLR::ALL::SET);

        self.regs.Timing0.set(TIMING0_640X480);
        self.regs.Timing1.set(TIMING1_640X480);
        self.regs.Timing2.set(TIMING2_640X480);
        self.regs.Timing3.set(TIMING3_640X480);

        self.regs.UPBASE.set(self.framebuffer as u32);
        self.regs.LPBASE.set(0);

        self.regs.CONTROL.write(
            CONTROL::LCDBPP::Bpp16
                + CONTROL::LCDTFT::SET
                + CONTROL::LCDPWR::SET
                + CONTROL::LCDVCOMP::VerticalBackPorch
                + CONTROL::LCDEN::SET,
        );
    }

    pub fn clear(&self, color: Rgb565) {
        for offset in 0..PIXELS {
            unsafe {
                ptr::write_volatile(self.framebuffer.add(offset), color.raw());
            }
        }
    }

    pub fn put_pixel(&self, x: usize, y: usize, color: Rgb565) {
        if let Some(offset) = pixel_offset(x, y) {
            unsafe {
                ptr::write_volatile(self.framebuffer.add(offset), color.raw());
            }
        }
    }

    pub fn fill_rect(&self, x: usize, y: usize, width: usize, height: usize, color: Rgb565) {
        let max_x = x.saturating_add(width).min(WIDTH);
        let max_y = y.saturating_add(height).min(HEIGHT);

        for py in y..max_y {
            let row = py * WIDTH;
            for px in x..max_x {
                unsafe {
                    ptr::write_volatile(self.framebuffer.add(row + px), color.raw());
                }
            }
        }
    }

    pub fn draw_char(&self, x: usize, y: usize, ch: char, foreground: Rgb565, background: Rgb565) {
        self.draw_char_scaled(x, y, ch, foreground, background, TEXT_SCALE);
    }

    pub fn draw_char_scaled(
        &self,
        x: usize,
        y: usize,
        ch: char,
        foreground: Rgb565,
        background: Rgb565,
        scale: usize,
    ) {
        let scale = scale.max(1);
        let glyph = glyph_for(ch);

        self.fill_rect(
            x,
            y,
            (FONT_WIDTH + 1) * scale,
            (FONT_HEIGHT + 1) * scale,
            background,
        );

        for row in 0..FONT_HEIGHT {
            let bits = glyph[row];
            for col in 0..FONT_WIDTH {
                if bits & (1 << (FONT_WIDTH - 1 - col)) != 0 {
                    self.fill_rect(x + col * scale, y + row * scale, scale, scale, foreground);
                }
            }
        }
    }

    pub fn draw_str(
        &self,
        mut x: usize,
        mut y: usize,
        text: &str,
        foreground: Rgb565,
        background: Rgb565,
    ) {
        let start_x = x;

        for ch in text.chars() {
            match ch {
                '\n' => {
                    x = start_x;
                    y = y.saturating_add(TEXT_LINE_HEIGHT);
                }
                '\r' => x = start_x,
                _ => {
                    if x + TEXT_CHAR_WIDTH > WIDTH {
                        x = start_x;
                        y = y.saturating_add(TEXT_LINE_HEIGHT);
                    }
                    if y + TEXT_LINE_HEIGHT > HEIGHT {
                        break;
                    }

                    self.draw_char(x, y, ch, foreground, background);
                    x = x.saturating_add(TEXT_CHAR_WIDTH);
                }
            }
        }
    }

    pub fn text_writer(&self, foreground: Rgb565, background: Rgb565) -> TextWriter {
        TextWriter::new(*self, foreground, background)
    }

    pub fn draw_test_pattern(&self) {
        self.clear(Rgb565::BLACK);
        self.fill_rect(0, 0, WIDTH / 3, HEIGHT, Rgb565::RED);
        self.fill_rect(WIDTH / 3, 0, WIDTH / 3, HEIGHT, Rgb565::GREEN);
        self.fill_rect((WIDTH / 3) * 2, 0, WIDTH / 3, HEIGHT, Rgb565::BLUE);
        self.fill_rect(40, 40, WIDTH - 80, 80, Rgb565::WHITE);
        self.fill_rect(60, 60, WIDTH - 120, 40, Rgb565::from_rgb(255, 200, 0));

        for offset in 0..HEIGHT.min(WIDTH) {
            self.put_pixel(offset, offset, Rgb565::WHITE);
            self.put_pixel(WIDTH - 1 - offset, offset, Rgb565::WHITE);
        }
    }
}

#[allow(dead_code)]
impl TextWriter {
    pub fn new(lcd: Lcd, foreground: Rgb565, background: Rgb565) -> Self {
        Self {
            lcd,
            cursor_x: 0,
            cursor_y: 0,
            foreground,
            background,
        }
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(WIDTH.saturating_sub(TEXT_CHAR_WIDTH));
        self.cursor_y = y.min(HEIGHT.saturating_sub(TEXT_LINE_HEIGHT));
    }

    pub fn set_colors(&mut self, foreground: Rgb565, background: Rgb565) {
        self.foreground = foreground;
        self.background = background;
    }

    pub fn clear(&mut self) {
        self.lcd.clear(self.background);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn put_char(&mut self, ch: char) {
        match ch {
            '\n' => self.newline(),
            '\r' => self.cursor_x = 0,
            '\t' => {
                for _ in 0..4 {
                    self.put_char(' ');
                }
            }
            _ => {
                if self.cursor_x + TEXT_CHAR_WIDTH > WIDTH {
                    self.newline();
                }

                if self.cursor_y + TEXT_LINE_HEIGHT > HEIGHT {
                    self.clear();
                }

                self.lcd.draw_char(
                    self.cursor_x,
                    self.cursor_y,
                    ch,
                    self.foreground,
                    self.background,
                );
                self.cursor_x = self.cursor_x.saturating_add(TEXT_CHAR_WIDTH);
            }
        }
    }

    pub fn puts(&mut self, text: &str) {
        for ch in text.chars() {
            self.put_char(ch);
        }
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = self.cursor_y.saturating_add(TEXT_LINE_HEIGHT);

        if self.cursor_y + TEXT_LINE_HEIGHT > HEIGHT {
            self.clear();
        }
    }
}

impl fmt::Write for TextWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);
        Ok(())
    }
}

pub fn init_printing(lcd: Lcd, foreground: Rgb565, background: Rgb565) {
    let writer = TextWriter::new(lcd, foreground, background);

    unsafe {
        ptr::addr_of_mut!(PRINT_WRITER).write(MaybeUninit::new(writer));
        PRINT_WRITER_READY = true;
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    unsafe {
        if PRINT_WRITER_READY {
            let writer = (*ptr::addr_of_mut!(PRINT_WRITER)).assume_init_mut();
            let _ = fmt::Write::write_fmt(writer, args);
        }
    }
}

#[macro_export]
macro_rules! lcd_print {
    ($($arg:tt)*) => {
        $crate::lcd::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! lcd_println {
    () => {
        $crate::lcd_print!("\n")
    };
    ($fmt:expr) => {
        $crate::lcd_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::lcd_print!(concat!($fmt, "\n"), $($arg)*)
    };
}

fn glyph_for(ch: char) -> [u8; FONT_HEIGHT] {
    let ch = if ch.is_ascii_lowercase() {
        ch.to_ascii_uppercase()
    } else {
        ch
    };

    match ch {
        ' ' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '"' => [
            0b01010, 0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '#' => [
            0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010,
        ],
        '$' => [
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ],
        '%' => [
            0b11000, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b00011,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '*' => [
            0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        ':' => [
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
        ],
        ';' => [
            0b00000, 0b01100, 0b01100, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '<' => [
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '>' => [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        '@' => [
            0b01110, 0b10001, 0b00001, 0b01101, 0b10101, 0b10101, 0b01110,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        '\\' => [
            0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '^' => [
            0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '`' => [
            0b01000, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '{' => [
            0b00010, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00010,
        ],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '}' => [
            0b01000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b01000,
        ],
        '~' => [
            0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000,
        ],
        _ => glyph_for('?'),
    }
}

#[allow(dead_code)]
const fn pixel_offset(x: usize, y: usize) -> Option<usize> {
    if x < WIDTH && y < HEIGHT {
        Some(y * WIDTH + x)
    } else {
        None
    }
}

fn framebuffer_base() -> *mut u16 {
    ptr::addr_of_mut!(FRAMEBUFFER).cast::<u16>()
}
