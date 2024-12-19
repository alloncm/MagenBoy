// Enabling all the features of the cortex-a72, currently rust has a bug preventing enabling with rustc 
// For more see - https://github.com/rust-lang/rust/issues/127269
.cpu cortex-a72

// alias the PERIPHERALS_BASE_ADDRESS symbol at pointer
// since it is a static global variable from the rust side
.equ    PERIPHERALS_BASE_ADDRESS_PTR,   PERIPHERALS_BASE_ADDRESS

.macro ldsctlr reg
    mrc     p15, 0, \reg, c1, c0, 0 // Read SCTLR
.endm
.macro stsctlr reg
    mcr     p15, 0, \reg, c1, c0, 0 // Write SCTLR
.endm

// source - https://developer.arm.com/documentation/ddi0406/c/System-Level-Architecture/System-Control-Registers-in-a-VMSA-implementation/VMSA-System-control-registers-descriptions--in-register-order/SCTLR--System-Control-Register--VMSA?lang=en
.equ    SCTLR_MMU_ENABLE,               (1<<0)
.equ    SCTLR_ENABLE_DATA_CACHE,        (1<<2)
.equ    SCTLR_ENABLE_BRANCH_PREDICTION, (1<<11) // Does not always exist, if implementation does not support BP or does not support disabling it
.equ    SCTLR_ENABLE_INSTRUCTION_CACHE, (1<<12)
.equ    SCTLR_ENABLE_AFE,               (1<<29)

// source - https://developer.arm.com/documentation/ddi0488/d/system-control/aarch64-register-descriptions/cpu-extended-control-register--el1
.equ    CPUECTLR_SMPEN,                 (1<<6)

// source - https://developer.arm.com/documentation/ddi0406/c/System-Level-Architecture/The-System-Level-Programmers--Model/ARM-processor-modes-and-ARM-core-registers/ARM-processor-modes?lang=en#CIHGHDGI
.equ    ARM_SUPERVISOR_MODE,            0x13
.equ    ARM_HYPERVISOR_MODE,            0x1A

// source - https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Advanced-SIMD-and-floating-point-support/The-Floating-Point-Exception-Register--FPEXC-?lang=en
.equ    FPEXC_VFP_SIMD_ENABLE,          (1<<30)

// source - https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Advanced-SIMD-and-floating-point-support/Enabling-Advanced-SIMD-and-floating-point-support
.equ    CPACR_CP10_CP11_ENABLE,         (0xF<<20) // Enable floating point and SIMD

// source - https://developer.arm.com/documentation/den0013/d/The-Memory-Management-Unit/First-level-address-translation
.equ    TTB_SECTION_NORMAL,             0x15C06 // normal memory TTB config
.equ    TTB_SECTION_DEVICE,             0x10C02 // device memory TTB config
.equ    L1_MEMORY_MAP_SIZE,             0x1000  // 4096 u32's
.equ    NORMAL_MAP_SIZE,                __cached_data_map_size // __uncached_data must by aligned to 1MB     

.section .text._start
.global _start
_start:
    mrc     p15, 0, r0, c0, c0, 5   // read MPIDR, 
    and     r0, r0, #0b11           // on coretx a72 those bits are the CPU core id
    cmp     r0, #0                  // check for cpu 0
    beq    .main_core
.park_loop:
    wfe
    b       .park_loop
.main_core:
// Setup ISR table
    ldr     r0, =isr_table
    ldr     r1, =__isr_table_addr
    mcr     p15, 0, r1, c12, c0, 0  // Write VBAR
    // copy the table 
    ldmia   r0!, {{r2,r3,r4,r5,r6,r7,r8,r9}}
    stmia   r1!, {{r2,r3,r4,r5,r6,r7,r8,r9}}
    ldmia   r0!, {{r2,r3,r4,r5,r6,r7,r8,r9}}
    stmia   r1!, {{r2,r3,r4,r5,r6,r7,r8,r9}}

// Enable HW floating point
    mrc     p15, 0, r0, c1, c0, 2   // Read CPACR
    orr     r0, r0, #CPACR_CP10_CP11_ENABLE
    mcr     p15, 0, r0, c1, c0, 2   // Write CPACR
    vmrs    r0, fpexc               // Read FPEXC 
    orr     r0, r0, #FPEXC_VFP_SIMD_ENABLE
    vmsr    fpexc, r0               // Write FPEXC

// Switch to supervisor mode
    mrs     r0, cpsr                // read current status register
    mov     r4, #0x1F               // CPSR mode mask
    and     r1, r0, r4              // get mode bits
    cmp     r1, #ARM_HYPERVISOR_MODE
    bne     .finis_mode_switch
    // On cortex a-72 you must enable CPUECTLR.SMPEN bit before touching the mmu and its accesible only in HYPERVISOR mdoe
    mrrc    p15, 1, r2, r3, c15         // Read CPUECTLR
    orr     r2, r2, #CPUECTLR_SMPEN
    mcrr    p15, 1, r2, r3, c15         // Write CPUECTLR
    // Switch to Supervisor mode
    bic     r0, r0, r4              // clear mode bits
    orr     r0, r0, #ARM_SUPERVISOR_MODE
    msr     spsr_cxsf, r0           // see msr docs - https://developer.arm.com/documentation/ddi0406/c/System-Level-Architecture/System-Instructions/Alphabetical-list-of-instructions/MSR--register-?lang=en
                                    // Also see this SO - https://stackoverflow.com/questions/15641149/current-program-status-register-exception-modes
    ldr     r0, =.finis_mode_switch // hold (in ELR_hyp) the address to return to  (to make 'eret' working right)
    msr     ELR_hyp, r0             // save the address in ELR_hyp
    eret                            // apply the mode change (Exception return)

.finis_mode_switch:
// Invalidate L1 cache
    // Disable cache and MMU
    ldsctlr r0
    bic     r0, r0, #SCTLR_ENABLE_DATA_CACHE
    bic     r0, r0, #SCTLR_ENABLE_INSTRUCTION_CACHE
    bic     r0, r0, #SCTLR_MMU_ENABLE
    stsctlr r0

    // invalidate I cache, TLB, branch predictor
    mov     r0, #0
    mcr     p15, 0, r0, c7, c5, 0   // Invalidate I cache
    mcr     p15, 0, r0, c7, c5, 6   // Invalidate branch prediction
    mcr     p15, 0, r0, c8, c7, 0   // Invalidate TLB

    // invalidate D cache
    // This is a machine idependent code to invalidate the D cache,
    // On most of the new armv7a and armv8a there is not need for this
    // and the CPU will boot just fine without it, but I want this code to portable so Im including this
    // and trying to document the hell out of it
    mov     r0, #0                  // Select L1 cache
    mcr     p15, 2, r0, c0, c0, 0   // Write CSSELR 
    mrc     p15, 1, r0, c0, c0, 0   // Read CCSIDR
    and     r1, r0, #0x7            // Extract line size (3bit), value is (log2(number_of_words)) - 4 (Some old docs are wrong and says its 2)
    add     r1, r1, #4              // add 4 to make r1 the number of bits to shift to get the number of bytes in line, 
                                    // thats how we calculate the offset of set om tje DCCISW command
    mov     r2, #0x7FFF             // mask to get the number of lines in cache
    and     r3, r2, r0, lsr #13     // r3 = r2 & (r0 << 13) -> lines_in_cache - 1
    mov     r2, #0x3FF              // mask to get the cache associativity
    and     r2, r2, r0, lsr #3      // r2 &= r0 << 3 -> associativity - 1
    clz     r4, r2                  // calc the leading zeros in u32 bits, this will indicate where to place the way param in the DCCISW command
    mov     r5, #0                  // way loop counter
.way_loop:
    mov     r6, #0                  // set loop counter
.set_loop:
    orr     r7, r0, r5, lsl r4      // set way
    orr     r7, r7, r6, lsl r1      // set set
    mcr     p15, 0, r7, c7, c6, 2   // Write DCCISW
    add     r6, r6, #1              // inc set
    cmp     r6, r3                  // check for last set
    ble     .set_loop               // if not iter set_loop
    add     r5, r5, #1              // else next way
    cmp     r5, r2                  // check last way
    ble     .way_loop               // if not iter way_loop

    // sync barriers after the invalidations and clearing
    dsb
    isb

// initialize translation tables
    mov     r0, 0                   // use short descriptor, base address is 16kb alligned
    mcr     p15, 0, r0, c2, c0, 2   // Write TTBCR

    ldr     r0, =0x55555555         // set all domains as clients (accesses are checked againt the translation table)
    mcr     p15, 0, r0, c3, c0, 0   // Set DACR

    ldsctlr r0
    bic     r0, r0, #SCTLR_ENABLE_AFE
    stsctlr r0

    ldr     r0, =l1_memory_map_1t1_table
    mov     r1, #0x2B               // rest of the config for TTBR0
    orr     r1, r0, r1              // combine the 16KB alligned address with the rest of the config
    mcr     p15, 0, r1, c2, c0, 0   // Write TTBR0

    //set up memory map
    mov     r1, #0                      // map start index
    ldr     r2, =TTB_SECTION_NORMAL
    ldr     r3, =NORMAL_MAP_SIZE
    bl      init_mmu_map_section        // init normal memory
    ldr     r2, =TTB_SECTION_DEVICE
    ldr     r3, =L1_MEMORY_MAP_SIZE     // the rest of the map is for device memory
    bl      init_mmu_map_section        // init device memory

// enable MMU and caches
    ldsctlr r0
    // the cortex a72 does not have an enable BP, but other cpus might have
    orr     r0, r0, #SCTLR_ENABLE_DATA_CACHE
    orr     r0, r0, #SCTLR_ENABLE_INSTRUCTION_CACHE
    orr     r0, r0, #SCTLR_MMU_ENABLE
    stsctlr r0
    dsb
    isb

// clear bss
    ldr     r0, =__bss_start
    ldr     r1, =__bss_end
    mov     r2, #0
.init_bss_loop:
    cmp     r0, r1
    beq    .finish_bss
    str     r2, [r0], #4            // increment r0 by 4 (dword)
    b       .init_bss_loop
.finish_bss:
// setting up the stack
    ldr     sp, =__cpu0_stack_start
    b       main

// Corrupts r4, r1 = end index
// r0 - L1 map ptr
// r1 - map start index
// r2 - ttb section configuration
// r3 - map end index
init_mmu_map_section:
    mov     r4, r1, lsl #20             // move r1 shifted left 20 to r3
    orr     r4, r4, r2                  // r4 = map section
    str     r4, [r0, r1, lsl #2]        // store r2 to r0 + (r1 * 4)
    add     r1, r1, #1                  // increment current index
    cmp     r1, r3                      // test current index vs end index
    blt     init_mmu_map_section        // loop if current index < end index
    bx      lr                          // return to caller

.global hang_led
hang_led:
    ldr     r3, =PERIPHERALS_BASE_ADDRESS_PTR
    ldr     r0, [r3]                // Deref the ptr to get the peripherals address
    ldr     r2, =0x200008           // GPFSEL2 MMIO register offset from base address
    add     r2, r2, r0              // add both to create GPFSEL2 address
    mov     r1, #1<<3               // Pin 21 as output mode
    str     r1, [r2]                // write to register
    ldr     r2, =0x20001C           // GPSET0 MMIO register offset from base address
    add     r2, r2, r0              // add both to create GPSET0 address
    mov     r1, #1<<21              // Pin 21 offset
    str     r1, [r2]                // Write to register
.hang_loop:
    wfe                             // easy on the busy waiting              
    b       .hang_loop              // verify no return

// interrupts jump table (ISR TABLE)
isr_table:    
    ldr     pc, _reset_h
    ldr     pc, _undefined_instruction_vector_h
    ldr     pc, _software_interrupt_vector_h
    ldr     pc, _prefetch_abort_vector_h
    ldr     pc, _data_abort_vector_h
    ldr     pc, _unused_handler_h
    ldr     pc, _interrupt_vector_h
    ldr     pc, _fast_interrupt_vector_h

_reset_h:                           .word   hang_led
_undefined_instruction_vector_h:    .word   hang_led
_software_interrupt_vector_h:       .word   hang_led
_prefetch_abort_vector_h:           .word   hang_led
_data_abort_vector_h:               .word   hang_led
_unused_handler_h:                  .word   hang_led
_interrupt_vector_h:                .word   hang_led
_fast_interrupt_vector_h:           .word   hang_led


.section .data
// This table must aligned for 16k address
.balign 16384
l1_memory_map_1t1_table:
    .rept L1_MEMORY_MAP_SIZE
    .word 0
    .endr