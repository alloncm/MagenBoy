use core::arch::asm;

const CNTP_CTL_ENABLE:u32   = 1;
const CNTP_CTL_IMASK:u32    = 1<<1;
const CNTP_CTL_ISTATUS:u32  = 1<<2;

// This function is busy waiting, when the interrupt controller will be implemented 
// I could make this function not busy wait
pub fn wait_ms(duration:u32){
    unsafe{
        let mut counter_timer_freq_reg:u32;                 // the freq of the system timer
        // read CNTFRQ ARM register
        asm!("mrc p15, 0, {r}, c14, c0, 0", r = out(reg) counter_timer_freq_reg);
    
        let required_count_increase:u32 = (counter_timer_freq_reg / 1000) * duration;
        // set CNTP_TVAL ARM register
        asm!("mcr p15, 0, {r}, c14, c2, 0", r = in(reg) required_count_increase);
        // set CNTP_CTL ARM register to start and disable timer interrupt
        let mut cntp_ctl_reg:u32;
        // read CNTP_CTL ARM register
        asm!("mrc p15, 0, {r}, c14, c2, 1", r = out(reg) cntp_ctl_reg);
        cntp_ctl_reg |= CNTP_CTL_ENABLE | CNTP_CTL_IMASK;
        asm!("mcr p15, 0, {r}, c14, c2, 1", r = in(reg) cntp_ctl_reg);
        loop {
            let mut cntp_ctl_reg:u32;
            asm!("mrc p15, 0, {r}, c14, c2, 1", r = out(reg) cntp_ctl_reg);
            if cntp_ctl_reg & CNTP_CTL_ISTATUS != 0{
                break;
            }
        }
        // set ARM Timer to stop counting
        let mut cntp_ctl_reg:u32;
        // read CNTP_CTL ARM register
        asm!("mrc p15, 0, {r}, c14, c2, 1", r = out(reg) cntp_ctl_reg);
        cntp_ctl_reg &= !CNTP_CTL_ENABLE;
        // set CNTP_CTL ARM register
        asm!("mcr p15, 0, {r}, c14, c2, 1", r = in(reg) cntp_ctl_reg);
    }
}