use drivers::keymaps::{Keymap, GB};
use spin::Mutex;
use x86_64::instructions::port::inb;

struct KeyPair {
    left: bool,
    right: bool,
}

impl KeyPair {
    const fn new() -> Self {
        KeyPair {
            left: false,
            right: false,
        }
    }

    fn is_pressed(&self) -> bool {
        self.left || self.right
    }
}

struct Modifiers {
    shift: KeyPair,
    control: KeyPair,
    alt: KeyPair,
    caps_lock: bool,
}

impl Modifiers {
    const fn new() -> Self {
        Modifiers {
            shift: KeyPair::new(),
            control: KeyPair::new(),
            alt: KeyPair::new(),
            caps_lock: false,
        }
    }

    fn use_uppercase_letters(&self) -> bool {
        self.shift.is_pressed() ^ self.caps_lock
    }

    fn apply_modifiers(&self, ascii: u8) -> u8 {
        if b'a' <= ascii && ascii <= b'z' {
            if self.use_uppercase_letters() {
                return ascii - b'a' + b'A';
            }
        } else if b'\\' == ascii {
            if self.use_uppercase_letters() {
                return b'|';
            }
        }
        ascii
    }

    fn update(&mut self, scancode: u8) {
        match scancode {
            0x1D => self.control.left = true,
            0x2A => self.shift.left = true,
            0x36 => self.shift.right = true,
            0x38 => self.alt.left = true,
            0x3A => self.caps_lock = !self.caps_lock,
            0x9D => self.control.left = false,
            0xAA => self.shift.left = false,
            0xB6 => self.shift.right = false,
            0xB8 => self.alt.left = false,
            _ => {}
        }
    }
}

struct Keyboard {
    modifiers: Modifiers,
    keymap: Keymap,
}

impl Keyboard {
    fn register_keypress(&mut self, scancode: u8) -> char {
        if self.modifiers.use_uppercase_letters() {
            if self.modifiers.shift.is_pressed() {
                self.keymap.shift_chars[scancode as usize]
            } else {
                self.keymap.caps_chars[scancode as usize]
            }
        } else {
            self.keymap.chars[scancode as usize]
        }
    }
}

static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    modifiers: Modifiers::new(),
    keymap: GB,
});

pub fn read_scancode_from_keyboard() -> Option<char> {
    let mut keyboard = KEYBOARD.lock();

    let scancode: u8;
    unsafe {
        scancode = inb(0x60);
    };
    keyboard.modifiers.update(scancode);
    if scancode < 0x80 {
        let ascii = keyboard.register_keypress(scancode);
        if !ascii.is_control() {
            return Some(ascii as char);
        } else if (ascii == '\x7F') || (ascii == '\n') {
            return (Some(ascii));
        } else {
            None
        }
    } else {
        None
    }
}
