use cpuio::{UnsafePort, Port};

// Command sent to begin PIC initialization.
const CMD_INIT: u8 = 0x11;

// Command sent to acknowledge an interrupt.
const CMD_END_OF_INTERRUPT: u8 = 0x20;

// The mode in which we want to run our PICs.
const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command: UnsafePort<u8>,
    data: UnsafePort<u8>,
}

impl Pic {
    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }

    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: UnsafePort::new(0x20),
                    data:    UnsafePort::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: UnsafePort::new(0xA0),
                    data:    UnsafePort::new(0xA1),
                },
            ]
        }
    }

    pub unsafe fn init(&mut self) {
        /// remap the PICs so they know their offset and master/slave wiring etc.
        let mut wait_port: Port<u8> = Port::new(0x80); // Its harmless to write 0 to 0x80,
                                                       // but it takes a little bit of time
                                                       // so we can use it to create a wait
                                                       // without a clock.
        let mut wait = || { wait_port.write(0) };      // Wait closure.

        let saved_mask_1 = self.pics[0].data.read();   // Save the masks so we don't have to
        let saved_mask_2 = self.pics[1].data.read();   // guess sensible values.

        self.pics[0].command.write(CMD_INIT);          // Write the init command to each PIC.
        wait();                                        // Wait in between to allow the message
        self.pics[1].command.write(CMD_INIT);          // time to be read.
        wait();

        self.pics[0].data.write(self.pics[0].offset);  // Write the information required to
        wait();                                        // set things up.
        self.pics[1].data.write(self.pics[1].offset);
        wait();
        self.pics[0].data.write(4);                    // 4 is the location of PIC2 (the slave)
        wait();                                        // it corresponds to IRQ2 (0000 0100)
        self.pics[1].data.write(2);                    // 2 tells PIC2 (the slave) its cascade
        wait();                                        // ID (000 0010)

        self.pics[0].data.write(MODE_8086);
        wait();
        self.pics[1].data.write(MODE_8086);
        wait();

        println!("{:b}\n{:b}\n", saved_mask_1, saved_mask_2);
        self.pics[0].data.write(saved_mask_1);
        self.pics[1].data.write(saved_mask_2);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}
