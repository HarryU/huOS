use spin::Mutex;
use x86_64::instructions::port::inb;

struct KeyPair {
    left:  bool,
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
    shift:     KeyPair,
    control:   KeyPair,
    alt:       KeyPair,
    caps_lock: bool,
}

impl Modifiers {
    const fn new() -> Self {
        Modifiers {
            shift:     KeyPair::new(),
            control:   KeyPair::new(),
            alt:       KeyPair::new(),
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
            _    => {},
        }
    }
}

struct Keyboard {
    modifiers: Modifiers,
}

static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard {
    modifiers: Modifiers::new(),
});

pub fn read_scancode_from_keyboard() -> Option<char> {
    let mut keyboard = KEYBOARD.lock();

    let scancode: u8;
    unsafe { scancode = inb(0x60); };

    keyboard.modifiers.update(scancode);
    if let Some(ascii) = match_ascii_scancode(scancode) {
        Some(keyboard.modifiers.apply_modifiers(ascii) as char)
    } else {
        None
    }
}

fn match_ascii_scancode(scancode: u8) -> Option<u8> {
    let idx = scancode as usize;
    let character = match scancode {
        0x01 ... 0x0E => Some(b"\x1B1234567890-=\0x02"[idx-0x01]),
        0x0F ... 0x1C => Some(b"\tqwertyuiop[]\r"[idx-0x0F]),
        0x1E ... 0x28 => Some(b"asdfghjkl;'"[idx-0x1E]),
        0x2C ... 0x35 => Some(b"zxcvbnm,./"[idx-0x2C]),
        0x39 => Some(b' '),
        _ => None,
    };
    character
}
