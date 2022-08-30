pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0 
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; REGISTER_COUNT],
    i_register: u16,
    stack_ptr: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; REGISTER_COUNT],
            i_register: 0,
            stack_ptr: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0
        }
    }
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Emulator::default();
        emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emulator
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; REGISTER_COUNT];
        self.i_register = 0;
        self.stack_ptr = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }

            self.sound_timer -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;

        self.pc += 2;
        op
    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_ptr as usize] = val;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_ptr -=1;
        self.stack[self.stack_ptr as usize]
    }

    // Instructions

    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn end_subroutine(&mut self) {
        let ret_addr = self.pop();
        self.pc = ret_addr;
    }

    fn jump(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn call_subroutine(&mut self, nnn: u16) {
        self.push(self.pc);
        self.pc = nnn;
    }

    fn skip_if_vx_equals_nn(&mut self, second_digit: u16, nn: u16) {
        let x = second_digit as usize;
        let nn = nn as u8;
        
        if self.v_reg[x] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_vx_not_equals_nn(&mut self, second_digit: u16, nn: u16) {
        let x = second_digit as usize;
        let nn = nn as u8;

        if self.v_reg[x] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_vx_equals_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    fn assign_nn_to_vx(&mut self, second_digit: u16, nn: u16) {
        let x = second_digit as usize;
        let nn = nn as u8;

        self.v_reg[x] = nn;
    }

    fn add_nn_to_vx(&mut self, second_digit: u16, nn: u16) {
        let x = second_digit as usize;
        let nn = nn as u8;

        self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
    }

    fn assign_vx_to_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        self.v_reg[x] = self.v_reg[y];
    }

    fn vx_or_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        self.v_reg[x] |= self.v_reg[y];
    }

    fn vx_and_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        self.v_reg[x] &= self.v_reg[y];
    }

    fn vx_xor_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        self.v_reg[x] ^= self.v_reg[y];
    }

    fn add_vy_to_vx(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;
        let vy = self.v_reg[y];

        let (vx, carry) = self.v_reg[x].overflowing_add(vy);
        let vf = if carry { 1 } else { 0 };

        self.v_reg[x] = vx;
        self.v_reg[0xF] = vf;
    }

    fn sub_vy_from_vx(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;
        let vy = self.v_reg[y];

        let (vx, carry) = self.v_reg[x].overflowing_sub(vy);
        let vf = if carry { 0 } else { 1 };

        self.v_reg[x] = vx;
        self.v_reg[0xF] = vf;
    }

    fn lshift_vx(&mut self, second_digit: u16) {
        let x = second_digit as usize;
        let msb = (self.v_reg[x] >> 7) & 1;

        self.v_reg[x] <<= 1;
        self.v_reg[0xF] = msb;
    }

    fn rshift_vx(&mut self, second_digit: u16) {
        let x = second_digit as usize;
        let lsb = self.v_reg[x] & 1;

        self.v_reg[x] >>= 1;
        self.v_reg[0xF] = lsb;
    }

    fn sub_vx_from_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;
        let vx = self.v_reg[x];

        let (vy, carry) = self.v_reg[y].overflowing_sub(vx);
        let vf = if carry { 0 } else { 1 };

        self.v_reg[y] = vy;
        self.v_reg[0xF] = vf;
    }

    fn skip_if_vx_not_equals_vy(&mut self, second_digit: u16, third_digit: u16) {
        let x = second_digit as usize;
        let y = third_digit as usize;

        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    fn execute(&mut self, op: u16) {
        let first_digit = (op & 0xF000) >> 12;
        let second_digit = (op & 0x0F00) >> 8;
        let third_digit = (op & 0x00F0) >> 4;
        let fourth_digit = op & 0x000F;

        let nnn = op & 0xFFF;
        let nn = op & 0xFF;

        match (first_digit, second_digit, third_digit, fourth_digit) {
            (0, 0, 0, 0) => (), // NOP
            (0, 0, 0xE, 0) => self.clear_screen(), // CLS
            (0, 0, 0xE, 0xE) => self.end_subroutine(), // RET
            (1, _, _, _) => self.jump(nnn), // JMP
            (2, _, _, _) => self.call_subroutine(nnn), // CALL
            (3, _, _, _) => self.skip_if_vx_equals_nn(second_digit, nn), // SKIP VX == NN
            (4, _, _, _) => self.skip_if_vx_not_equals_nn(second_digit, nn), // SKIP VX != NN
            (5, _, _, 0) => self.skip_if_vx_equals_vy(second_digit, third_digit), // SKIP VX == VY
            (6, _, _, _) => self.assign_nn_to_vx(second_digit, nn), // VX == NN
            (7, _, _, _) => self.add_nn_to_vx(second_digit, nn), // VX += NN
            (8, _, _, 0) => self.assign_vx_to_vy(second_digit, third_digit), // VX = VY
            (8, _, _, 1) => self.vx_or_vy(second_digit, third_digit), // VX = VY
            (8, _, _, 2) => self.vx_and_vy(second_digit, third_digit), // VX &= VY
            (8, _, _, 3) => self.vx_xor_vy(second_digit, third_digit), // VX ^= VY
            (8, _, _, 4) => self.add_vy_to_vx(second_digit, third_digit), // VX += VY
            (8, _, _, 5) => self.sub_vy_from_vx(second_digit, third_digit), // VX -= VY
            (8, _, _, 6) => self.rshift_vx(second_digit), // VX >>= 1
            (8, _, _, 7) => self.sub_vx_from_vy(second_digit, third_digit), // VX = VY - VX
            (8, _, _, 0xE) => self.lshift_vx(second_digit), // VX <<= 1
            (9, _, _, 0) => self.skip_if_vx_not_equals_vy(second_digit, third_digit),
            _ => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}
