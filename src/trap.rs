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
        medeleg::set_instruction_fault();
        medeleg::set_load_fault();
        medeleg::set_store_fault();
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
}

#[export_name = "_start_trap_rust"]
extern "C" fn start_trap_rust(trap_frame: &mut TrapFrame) {
    use crate::sys::ClintTimer;
    use riscv::register::{
        mcause::{self, Exception, Interrupt, Trap},
        mepc, mie, mip, mtval,
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
                mip::set_ssoft();
                mie::clear_msoft();
            }
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            unsafe {
                mip::set_stimer();
                mie::clear_mtimer();
            }
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            #[inline]
            unsafe fn get_vaddr_u32(vaddr: usize) -> u32 {
                let mut ans: u32;
                #[cfg(target_pointer_width = "64")]
                llvm_asm!("
                    li      t0, (1 << 17)
                    mv      t1, $1
                    csrrs   t0, mstatus, t0
                    lwu     t1, 0(t1)
                    csrw    mstatus, t0
                    mv      $0, t1
                "
                    :"=r"(ans) 
                    :"r"(vaddr)
                    :"t0", "t1");

                #[cfg(target_pointer_width = "32")]
                llvm_asm!("
                    li      t0, (1 << 17)
                    mv      t1, $1
                    csrrs   t0, mstatus, t0
                    lw     t1, 0(t1)
                    csrw    mstatus, t0
                    mv      $0, t1
                "
                    :"=r"(ans) 
                    :"r"(vaddr)
                    :"t0", "t1");

                ans
            }
            let vaddr = mepc::read();
            let ins = unsafe { get_vaddr_u32(vaddr) };
            if ins & 0xFFFFF07F == 0xC0102073 {
                // rdtime
                let rd = ((ins >> 7) & 0b1_1111) as u8;
                let time_usize = ClintTimer.get_time() as usize;
                match rd {
                    10 => trap_frame.a0 = time_usize,
                    11 => trap_frame.a1 = time_usize,
                    12 => trap_frame.a2 = time_usize,
                    13 => trap_frame.a3 = time_usize,
                    14 => trap_frame.a4 = time_usize,
                    15 => trap_frame.a5 = time_usize,
                    16 => trap_frame.a6 = time_usize,
                    17 => trap_frame.a7 = time_usize,
                    5 => trap_frame.t0 = time_usize,
                    6 => trap_frame.t1 = time_usize,
                    7 => trap_frame.t2 = time_usize,
                    28 => trap_frame.t3 = time_usize,
                    29 => trap_frame.t4 = time_usize,
                    30 => trap_frame.t5 = time_usize,
                    31 => trap_frame.t6 = time_usize,
                    _ => panic!("invalid target"),
                }
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
            "Unhandled exception! mcause: {:?}, mepc: {:016x?}, mtval: {:016x?}, trap frame: {:p}, {:x?}",
            cause,
            mepc::read(),
            mtval::read(),
            &trap_frame as *const _,
            trap_frame
        ),
        #[cfg(target_pointer_width = "32")]
        cause => panic!(
            "Unhandled exception! mcause: {:?}, mepc: {:08x?}, mtval: {:08x?}, trap frame: {:x?}",
            cause,
            mepc::read(),
            mtval::read(),
            trap_frame
        ),
    }
}
