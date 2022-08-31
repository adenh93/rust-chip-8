use rand::random;

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
    i_reg: u16,
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
            i_reg: 0,
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
        self.i_reg = 0;
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

    fn assign_nnn_to_ireg(&mut self, nnn: u16) {
        self.i_reg = nnn
    }

    fn jump_to_offset(&mut self, nnn: u16) {
        self.pc = (self.v_reg[0] as u16) + nnn;
    }

    fn assign_rand_and_nn_to_vx(&mut self, second_digit: u16, nn: u16) {
        let x = second_digit as usize; 
        let nn = nn as u8;
        let rng: u8 = random();

        self.v_reg[x] = rng & nn;
    }

    fn draw_sprite(&mut self, vx: u16, vy: u16, num_rows: u16) {
        let x_coord = self.v_reg[vx as usize] as u16;
        let y_coord = self.v_reg[vy as usize] as u16;
        
        let mut flipped = false;

        for y_line in 0..num_rows {
            let addr = self.i_reg + y_line as u16;
            let pixels = self.ram[addr as usize];

            for x_line in 0..8 {
                if (pixels & (0b10000000 >> x_line)) != 0 {
                    let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y_line) as usize & SCREEN_HEIGHT;

                    let idx = x + SCREEN_WIDTH * y;
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }

        self.v_reg[0xF] = flipped.into()
    }

    fn skip_if_key_pressed(&mut self, x: u16) {
        let vx = self.v_reg[x as usize];
        let key = self.keys[vx as usize];

        if key {
            self.pc += 2;
        }
    }

    fn skip_if_key_not_pressed(&mut self, x: u16) {
        let vx = self.v_reg[x as usize];
        let key = self.keys[vx as usize];

        if !key {
            self.pc += 2;
        }
    }

    fn assign_dt_to_vx(&mut self, x: u16) {
        let x = x as usize;
        self.v_reg[x] = self.delay_timer;
    }

    fn wait_for_key_press(&mut self, x: u16) {
        let x = x as usize;
        let mut pressed = false;

        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.pc -= 2;
        }
    }

    fn assign_vx_to_dt(&mut self, x: u16) {
        let vx = self.v_reg[x as usize];
        self.delay_timer = vx;
    }

    fn assign_vx_to_st(&mut self, x: u16) {
        let vx = self.v_reg[x as usize];
        self.sound_timer = vx;
    }

    fn add_vx_to_ireg(&mut self, x: u16) {
        let vx = self.v_reg[x as usize] as u16;
        self.i_reg = self.i_reg.wrapping_add(vx);
    }

    fn assign_font_addr_to_ireg(&mut self, x: u16) {
        let x = x as usize;
        let c = self.v_reg[x] as u16;
        self.i_reg = c * 5;
    }

    fn assign_vx_bcd_to_ireg(&mut self, x: u16) {
        let vx = self.v_reg[x as usize] as f32;

        let hundreds = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    fn store_regs_into_ram(&mut self, x: u16) {
        let x = x as usize;
        let i = self.i_reg as usize;

        for idx in 0..=x {
            self.ram[i + idx] = self.v_reg[idx];
        }
    }

    fn load_ram_into_regs(&mut self, x: u16) {
        let x = x as usize;
        let i = self.i_reg as usize;

        for idx in 0..=x {
            self.v_reg[idx] = self.ram[i + idx];
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
            (3, _, _, _) => self.skip_if_vx_equals_nn(second_digit, nn), // SE VX, NN
            (4, _, _, _) => self.skip_if_vx_not_equals_nn(second_digit, nn), // SNE VX, NN
            (5, _, _, 0) => self.skip_if_vx_equals_vy(second_digit, third_digit), // SE VX, VY
            (6, _, _, _) => self.assign_nn_to_vx(second_digit, nn), // VX == NN
            (7, _, _, _) => self.add_nn_to_vx(second_digit, nn), // VX += NN
            (8, _, _, 0) => self.assign_vx_to_vy(second_digit, third_digit), // VX = VY
            (8, _, _, 1) => self.vx_or_vy(second_digit, third_digit), // VX |= VY
            (8, _, _, 2) => self.vx_and_vy(second_digit, third_digit), // VX &= VY
            (8, _, _, 3) => self.vx_xor_vy(second_digit, third_digit), // VX ^= VY
            (8, _, _, 4) => self.add_vy_to_vx(second_digit, third_digit), // VX += VY
            (8, _, _, 5) => self.sub_vy_from_vx(second_digit, third_digit), // VX -= VY
            (8, _, _, 6) => self.rshift_vx(second_digit), // VX >>= 1
            (8, _, _, 7) => self.sub_vx_from_vy(second_digit, third_digit), // VX = VY - VX
            (8, _, _, 0xE) => self.lshift_vx(second_digit), // VX <<= 1
            (9, _, _, 0) => self.skip_if_vx_not_equals_vy(second_digit, third_digit), // SNE VX, VY
            (0xA, _, _, _) => self.assign_nnn_to_ireg(nnn), // I = NNN
            (0xB, _, _, _) => self.jump_to_offset(nnn), // JMP V0 + NNN
            (0xC, _, _, _) => self.assign_rand_and_nn_to_vx(second_digit, nn), // VX = RAND & NN
            (0xD, _, _, _) => self.draw_sprite(second_digit, third_digit, fourth_digit), // DRW
            (0xE, _, 9, 0xE) => self.skip_if_key_pressed(second_digit), // SKP
            (0xE, _, 0xA, 1) => self.skip_if_key_not_pressed(second_digit), //SKNP 
            (0xF, _, 0, 7) => self.assign_dt_to_vx(second_digit), // VX = DT
            (0xF, _, 0, 0xA) => self.wait_for_key_press(second_digit), // LD VX, K
            (0xF, _, 1, 5) => self.assign_vx_to_dt(second_digit), // LD DT, VX
            (0xF, _, 1, 8) => self.assign_vx_to_st(second_digit), // LD ST, VX
            (0xF, _, 1, 0xE) => self.add_vx_to_ireg(second_digit), // I += VX
            (0xF, _, 2, 9) => self.assign_font_addr_to_ireg(second_digit), // LD F, VX 
            (0xF, _, 3, 3) => self.assign_vx_bcd_to_ireg(second_digit), // LD B, VX
            (0xF, _, 5, 5) => self.store_regs_into_ram(second_digit), // LD [I], VX
            (0xF, _, 6, 6) => self.load_ram_into_regs(second_digit), // LD VX, [I]
            _ => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}
