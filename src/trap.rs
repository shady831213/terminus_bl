#[cfg(target_pointer_width = "32")]
global_asm!(include_str!("rv32.S"));

#[cfg(target_pointer_width = "64")]
global_asm!(include_str!("rv64.S"));

global_asm!(include_str!("trap.S"));
pub fn init_trap() {
    use riscv::register::mtvec::{self, TrapMode};
    extern "C" {
        fn _start_trap();
    }
    unsafe {
        mtvec::write(_start_trap as usize, TrapMode::Direct);
    }
}

pub fn delege_trap() {
    use riscv::register::{medeleg, mideleg, mie};
    unsafe {
        mideleg::set_sext();
        mideleg::set_stimer();
        mideleg::set_ssoft();
        medeleg::set_instruction_misaligned();
        medeleg::set_breakpoint();
        medeleg::set_user_env_call();
        medeleg::set_instruction_page_fault();
        medeleg::set_load_page_fault();
        medeleg::set_store_page_fault();
        mie::set_mext();
        mie::set_msoft();
    }
}

#[derive(Debug)]
struct TrapFrame {
    ra: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s0: usize,
    s1: usize,
}

impl TrapFrame {
    fn update(&mut self, dst: u8, value: usize) {
        match dst {
            8 => self.s0 = value,
            9 => self.s1 = value,
            10 => self.a0 = value,
            11 => self.a1 = value,
            12 => self.a2 = value,
            13 => self.a3 = value,
            14 => self.a4 = value,
            15 => self.a5 = value,
            16 => self.a6 = value,
            17 => self.a7 = value,
            5 => self.t0 = value,
            6 => self.t1 = value,
            7 => self.t2 = value,
            28 => self.t3 = value,
            29 => self.t4 = value,
            30 => self.t5 = value,
            31 => self.t6 = value,
            _ => panic!("invalid target{}", dst),
        }
    }
}

#[export_name = "_start_trap_rust"]
extern "C" fn start_trap_rust(trap_frame: &mut TrapFrame) {
    use crate::sys::ClintTimer;
    use riscv::register::{
        mcause::{self, Exception, Interrupt, Trap},
        mepc, mie, mip, mstatus, mtval,
    };
    let cause = mcause::read().cause();
    match cause {
        Trap::Exception(Exception::SupervisorEnvCall) => {
            let params = [trap_frame.a0, trap_frame.a1, trap_frame.a2, trap_frame.a3];
            // Call RustSBI procedure
            let ans = rustsbi::ecall(trap_frame.a7, trap_frame.a6, params);
            // Return the return value to TrapFrame
            trap_frame.a0 = ans.error;
            trap_frame.a1 = ans.value;
            // Skip ecall instruction
            mepc::write(mepc::read().wrapping_add(4));
        }
        Trap::Interrupt(Interrupt::MachineSoft) => {
            unsafe {
                mie::clear_msoft();
                mip::set_ssoft();
            }
        }
        Trap::Interrupt(Interrupt::MachineExternal) => {
            unsafe {
                mie::clear_mext();
                mip::set_sext();
            }
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            unsafe {
                mie::clear_mtimer();
                mip::set_stimer();
            }
        }
        // Trap::Exception(Exception::LoadMisaligned) => {
        //     let vaddr = mepc::read();
        //     let ins = unsafe { crate::sys::get_insn(vaddr) };
        // }
        Trap::Exception(Exception::IllegalInstruction) => {
            let vaddr = mepc::read();
            let ins = unsafe { crate::sys::get_insn(vaddr) };
            if ins & 0xFFFFF07F == 0xC0102073 {
                // rdtime
                let rd = ((ins >> 7) & 0b1_1111) as u8;
                let time_usize = ClintTimer.get_time() as usize;
                trap_frame.update(rd, time_usize);
                mepc::write(mepc::read().wrapping_add(4));
            } else {
                #[cfg(target_pointer_width = "64")]
                panic!("invalid instruction, mepc: {:016x?}, instruction: {:016x?}", mepc::read(), ins);
                #[cfg(target_pointer_width = "32")]
                panic!("invalid instruction, mepc: {:08x?}, instruction: {:08x?}", mepc::read(), ins);
            }
        }
        #[cfg(target_pointer_width = "64")]
        cause => panic!(
            "Unhandled exception! mstatus: {:x?}, mcause: {:?}, mepc: {:016x?}, mtval: {:016x?}, trap frame: {:p}, {:x?}",
            mstatus::read(),
            cause,
            mepc::read(),
            mtval::read(),
            &trap_frame as *const _,
            trap_frame
        ),
        #[cfg(target_pointer_width = "32")]
        cause => panic!(
            "Unhandled exception! mstatus: {:x?}, mcause: {:?}, mepc: {:08x?}, mtval: {:08x?}, trap frame: {:x?}",
            mstatus::read(),
            cause,
            mepc::read(),
            mtval::read(),
            trap_frame
        ),
    }
}
