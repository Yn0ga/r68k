pub type Handler = fn(&mut Core);
pub type InstructionSet = Vec<Handler>;
use ram::{LoggingMem, AddressBus, OpsLogger, SUPERVISOR_PROGRAM, USER_PROGRAM};
pub mod ops;

pub struct Core {
	pub pc: u32,
	pub inactive_ssp: u32, // when in user mode
	pub inactive_usp: u32, // when in supervisor mode
	pub ir: u16,
	pub dar: [u32; 16],
	pub ophandlers: InstructionSet,
	pub s_flag: u32,
	pub int_mask: u32,
	pub x_flag: u32,
	pub c_flag: u32,
	pub v_flag: u32,
	pub n_flag: u32,
	pub prefetch_addr: u32,
	pub prefetch_data: u32,
	pub not_z_flag: u32,

	pub mem: LoggingMem<OpsLogger>,
}

// these values are borrowed from Musashi
// and not yet fully understood
const SFLAG_SET: u32 =  0x04;
const XFLAG_SET: u32 = 0x100;
const NFLAG_SET: u32 =  0x80;
const VFLAG_SET: u32 =  0x80;
const CFLAG_SET: u32 = 0x100;
const CPU_SR_MASK: u32 = 0xa71f; /* T1 -- S  -- -- I2 I1 I0 -- -- -- X  N  Z  V  C  */
const CPU_SR_INT_MASK: u32 = 0x0700;

impl Core {
	pub fn new(base: u32) -> Core {
		Core { pc: base, prefetch_addr: 0, prefetch_data: 0, inactive_ssp: 0, inactive_usp: 0, ir: 0, s_flag: SFLAG_SET, int_mask: CPU_SR_INT_MASK, dar: [0u32; 16], mem: LoggingMem::new(0xffffffff, OpsLogger::new()), ophandlers: ops::fake::instruction_set(), x_flag: 0, v_flag: 0, c_flag: 0, n_flag: 0, not_z_flag: 0xffffffff}
	}
	pub fn new_mem(base: u32, contents: &[u8]) -> Core {
		let mut lm = LoggingMem::new(0xffffffff, OpsLogger::new());
		for (offset, byte) in contents.iter().enumerate() {
			lm.write_u8(base + offset as u32, *byte as u32);
		}
		Core { pc: base, prefetch_addr: 0, prefetch_data: 0, inactive_ssp: 0, inactive_usp: 0, ir: 0, s_flag: SFLAG_SET, int_mask: CPU_SR_INT_MASK, dar: [0u32; 16], mem: lm, ophandlers: ops::fake::instruction_set(), x_flag: 0, v_flag: 0, c_flag: 0, n_flag: 0, not_z_flag: 0xffffffff }
	}
	pub fn reset(&mut self) {
		self.s_flag = SFLAG_SET;
		self.int_mask = CPU_SR_INT_MASK;
		self.jump(0);
		self.dar[15] = self.read_imm_32();
		let new_pc = self.read_imm_32();
		self.jump(new_pc);
	}
	pub fn x_flag_as_1(&self) -> u32 {
		(self.x_flag>>8)&1
	}
	// admittely I've chosen to reuse Musashi's representation of flags
	// which I don't fully understand (they are not matching their
	// positions in the SR/CCR)
	pub fn status_register(&self) -> u32 {
		(self.s_flag << 11)                 |
		self.int_mask						|
		((self.x_flag & XFLAG_SET) >> 4)	|
		((self.n_flag & NFLAG_SET) >> 4)	|
		((not1!(self.not_z_flag))  << 2)	|
		((self.v_flag & VFLAG_SET) >> 6)	|
		((self.c_flag & CFLAG_SET) >> 8)
	}

	// admittely I've chosen to reuse Musashi's representation of flags
	// which I don't fully understand (they are not matching their
	// positions in the SR/CCR)
	pub fn sr_to_flags(&mut self, sr: u32) {
		let sr = sr & CPU_SR_MASK;
		self.int_mask = sr & CPU_SR_INT_MASK;
		self.s_flag =		   (sr >> 11) & SFLAG_SET;
		self.x_flag = 		   (sr <<  4) & XFLAG_SET;
		self.n_flag = 		   (sr <<  4) & NFLAG_SET;
		self.not_z_flag = not1!(sr & 0b00100);
		self.v_flag = 		   (sr <<  6) & VFLAG_SET;
		self.c_flag = 		   (sr <<  8) & CFLAG_SET;
		// println!("{} {:016b} {} {}", self.flags(), sr, self.not_z_flag, sr & 0b00100);
	}

	pub fn flags(&self) -> String {
		let sr = self.status_register();
		let supervisor = (sr >> 13) & 1;
		let irq_mask = (0x700 & sr) >> 8;

		format!("-{}{}{}{}{}{}{}",
		if supervisor > 0 {'S'} else {'U'},
		irq_mask,
		if 0 < (sr >> 4) & 1 {'X'} else {'-'},
		if 0 < (sr >> 3) & 1 {'N'} else {'-'},
		if 0 < (sr >> 2) & 1 {'Z'} else {'-'},
		if 0 < (sr >> 1) & 1 {'V'} else {'-'},
		if 0 < (sr     ) & 1 {'C'} else {'-'})
	}

	pub fn read_imm_32(&mut self) -> u32 {
		let b = self.pc;
		self.pc += 4;
		self.mem.read_long(SUPERVISOR_PROGRAM, b)
	}
	pub fn read_imm_16(&mut self) -> u16 {
		// the Musashi read_imm_16 calls cpu_read_long as part of prefetch
		if self.pc & 1 > 0 {
			panic!("Address error, odd PC at {:08x}", self.pc);
		}
		// prefetches are 4-byte-aligned
		if self.pc & !3 != self.prefetch_addr {
			self.prefetch_addr = self.pc & !3;
			let address_space = if self.s_flag != 0 {SUPERVISOR_PROGRAM} else {USER_PROGRAM};
			self.prefetch_data = self.mem.read_long(address_space, self.prefetch_addr);
		}
		self.pc += 2;
		((self.prefetch_data >> ((2 - ((self.pc - 2) & 2))<<3)) & 0xffff) as u16
	}
	pub fn jump(&mut self, pc: u32) {
		self.pc = pc;
	}
	pub fn execute1(&mut self) {
		// Read an instruction from PC (increments PC by 2)
		self.ir = self.read_imm_16();
		let opcode = self.ir as usize;

		// Call instruction handler to mutate Core accordingly
		self.ophandlers[opcode](self);

		// TODO: Perform CPU-cycle accounting for this instruction
	}
}

impl Clone for Core {
	fn clone(&self) -> Self {
		let mut lm = LoggingMem::new(0xffffffff, OpsLogger::new());
		lm.copy_from(&self.mem);
		assert_eq!(0, lm.logger.len());
		Core { pc: self.pc, prefetch_addr: 0, prefetch_data: 0, inactive_ssp: self.inactive_ssp, inactive_usp: self.inactive_usp, ir: self.ir, s_flag: self.s_flag, int_mask: self.int_mask, dar: self.dar, mem: lm, ophandlers: ops::instruction_set(), x_flag: self.x_flag, v_flag: self.v_flag, c_flag: self.c_flag, n_flag: self.n_flag, not_z_flag: self.not_z_flag}
	}
}

#[cfg(test)]
mod tests {
	use super::Core;
	use super::ops; //::instruction_set;
	use ram::{AddressBus, Operation, SUPERVISOR_PROGRAM, USER_PROGRAM};

	#[test]
	fn new_sets_pc() {
		let cpu = Core::new(256);
		assert_eq!(256, cpu.pc);
	}

	#[test]
	fn new_mem_sets_pc_and_mem() {
		let base = 128;
		let cpu = Core::new_mem(base, &[1u8, 2u8, 3u8, 4u8, 5u8, 6u8]);
		assert_eq!(128, cpu.pc);
		assert_eq!(1, cpu.mem.read_byte(SUPERVISOR_PROGRAM, 128));
		assert_eq!(2, cpu.mem.read_byte(SUPERVISOR_PROGRAM, 129));
	}

	#[test]
	fn a_jump_changes_pc() {
		let mut cpu = Core::new(0);
		cpu.jump(128);
		assert_eq!(128, cpu.pc);
	}

	#[test]
	fn an_imm_read32_changes_pc() {
		let base = 128;
		let mut cpu = Core::new(base);
		cpu.read_imm_32();
		assert_eq!(base+4, cpu.pc);
	}

	#[test]
	fn an_imm_read32_reads_from_pc() {
		let base = 128;
		let mut cpu = Core::new_mem(base, &[2u8, 1u8, 3u8, 4u8]);
		let val = cpu.read_imm_32();
		assert_eq!((2<<24)+(1<<16)+(3<<8)+4, val);
	}


	#[test]
	fn an_imm_read16_changes_pc() {
		let base = 128;
		let mut cpu = Core::new(base);
		cpu.read_imm_16();
		assert_eq!(base+2, cpu.pc);
	}

	#[test]
	fn an_imm_read16_reads_from_pc() {
		let base = 128;
		let mut cpu = Core::new_mem(base, &[2u8, 1u8, 3u8, 4u8]);
		assert_eq!("-S7-----", cpu.flags());

		let val = cpu.read_imm_16();
		assert_eq!((2<<8)+(1<<0), val);
		assert_eq!(Operation::ReadLong(SUPERVISOR_PROGRAM, base, 0x02010304), cpu.mem.logger.ops()[0]);
	}

	#[test]
	fn an_user_mode_imm_read16_is_reflected_in_mem_ops() {
		let base = 128;
		let mut cpu = Core::new_mem(base, &[2u8, 1u8, 3u8, 4u8]);
		cpu.s_flag = 0;
		assert_eq!("-U7-----", cpu.flags());

		let val = cpu.read_imm_16();
		assert_eq!((2<<8)+(1<<0), val);
		assert_eq!(Operation::ReadLong(USER_PROGRAM, base, 0x02010304), cpu.mem.logger.ops()[0]);
	}

	#[test]
	fn a_reset_reads_sp_and_pc_from_0() {
		let mut cpu = Core::new_mem(0, &[0u8,0u8,1u8,0u8, 0u8,0u8,0u8,128u8]);
		cpu.reset();
		assert_eq!(256, cpu.dar[15]);
		assert_eq!(128, cpu.pc);
		assert_eq!("-S7-----", cpu.flags());
		assert_eq!(Operation::ReadLong(SUPERVISOR_PROGRAM, 0, 0x100), cpu.mem.logger.ops()[0]);
	}

	#[test]
	#[should_panic(expected = "instruction bad1")]
	fn execute_reads_from_pc_and_panics_on_illegal_instruction() {
		let mut cpu = Core::new_mem(0xba, &[0xba,0xd1,1u8,0u8, 0u8,0u8,0u8,128u8]);
		cpu.execute1();
	}
	#[test]
	#[should_panic(expected = "Address error")]
	fn execute_panics_on_odd_pc() {
		let mut cpu = Core::new_mem(0xbd, &[0x00, 0x0a, 0x00, 0x00]);
		cpu.execute1();
	}

	#[test]
	fn execute_can_execute_instruction_handler_0a() {
		let mut cpu = Core::new_mem(0xba, &[0x00, 0x0A, 1u8,0u8, 0u8,0u8,0u8,128u8]);
		cpu.execute1();
		assert_eq!(0xabcd, cpu.dar[0]);
		assert_eq!(0x0000, cpu.dar[1]);
	}

	#[test]
	fn execute_can_execute_instruction_handler_0b() {
		let mut cpu = Core::new_mem(0xba, &[0x00, 0x0B, 1u8,0u8, 0u8,0u8,0u8,128u8]);
		cpu.execute1();
		assert_eq!(0x0000, cpu.dar[0]);
		assert_eq!(0xbcde, cpu.dar[1]);
	}


	#[test]
	fn execute_can_execute_set_dx() {
		// first byte 40 is register D0
		// 42 == D1
		// 44 == D2
		// 46 == D3
		// 48 == D4
		// 4a == D5
		// 4c == D6
		// 4e == D7
		let mut cpu = Core::new_mem(0x40, &[0x4c, 0x00, 1u8, 0u8]);
		cpu.execute1();
		assert_eq!(0xcdef, cpu.dar[6]);
	}

	#[test]
	fn array_elems() {
		let mut arr = [1, 2, 3, 4];
		let mut marr = &mut arr;
		let mut elem: &mut i32 = &mut (marr[1]);
		// let mut elem2: &mut i32 = &mut (arr[2]);
		assert_eq!(2, *elem);
		*elem = 200;
		assert_eq!(200, *elem);
		// assert_eq!(200, &mut marr[1]);
	}

	#[test]
	fn abcd_8_rr() {
		// opcodes c100 - c107, c300 - c307, etc.
		// or more generally c[13579bdf]0[0-7]
		// where [13579bdf] is DX (dest regno) and [0-7] is DY (src regno)
		// so c300 means D1 = D0 + D1 in BCD
		let mut cpu = Core::new_mem(0x40, &[0xc3, 0x00]);
		cpu.ophandlers = ops::instruction_set();

		cpu.dar[0] = 0x16;
		cpu.dar[1] = 0x26;
		cpu.execute1();

		// 16 + 26 is 42
		assert_eq!(0x42, cpu.dar[1]);
	}

	#[test]
	fn status_register_roundtrip(){
		let mut core = Core::new(0x40);
		//Status register bits are:
		//      TTSM_0iii_000X_NZVC;
		let f=0b0000_1000_1110_0000; // these bits should always be zero
		let s=0b0010_0000_0000_0000;
		let i=0b0000_0111_0000_0000;
		let x=0b0000_0000_0001_0000;
		let n=0b0000_0000_0000_1000;
		let z=0b0000_0000_0000_0100;
		let v=0b0000_0000_0000_0010;
		let c=0b0000_0000_0000_0001;
		let flags = vec![x,n,z,v,c,f,s,i,0];
		for sf in flags {
			core.sr_to_flags(sf);
			let sr = core.status_register();
			let expected = if sf == f {0} else {sf};
			assert_eq!(expected, sr);
		}
	}
	#[test]
	fn clones_have_independent_registers() {
		let mut core = Core::new(0x40);
		core.dar[1] = 0x16;
		let mut clone = core.clone();
		assert_eq!(0x16, core.dar[1]);
		assert_eq!(0x16, clone.dar[1]);
		clone.dar[1] = 0x32;
		assert_eq!(0x16, core.dar[1]);
		assert_eq!(0x32, clone.dar[1]);
	}
}