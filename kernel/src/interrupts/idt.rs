use super::apic;
use crate::timers;
use core::ops::RangeInclusive;
use x86_64::structures::idt::{ExceptionVector, InterruptDescriptorTable, InterruptStackFrame};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Fills IDT
pub fn init() {
    #[allow(static_mut_refs)]
    unsafe {
        x86_64::set_general_handler!(&mut IDT, general_interrupt_handler);
        // Loads IDT using lidt
        IDT.load();
    }
}

pub const CPU_EXCEPTIONS_IDT_VECTORS_RANGE: RangeInclusive<u8> = 0..=31;
pub const IO_APIC_ISA_IRQ_VECTORS_RANGE: RangeInclusive<u8> = 32..=47;
pub const IO_APIC_24_VECTORS_RANGE: RangeInclusive<u8> = 32..=55;
pub const LOCAL_APIC_TIMER_IDT_VECTOR: u8 = 56;
pub const LOCAL_APIC_LINT0_IDT_VECTOR: u8 = 57;
pub const LOCAL_APIC_LINT1_IDT_VECTOR: u8 = 58;
pub const LOCAL_APIC_ERROR_IDT_VECTOR: u8 = 59;
pub const LOCAL_APIC_SPURIOUS_IDT_VECTOR: u8 = 255;

/// A general handler function for an interrupt or an exception with the interrupt/exception index and an optional error code
///
/// 0-31    CPU exceptions<br>
/// 32-47   IO APIC Legacy ISA IRQ's
/// 32-55   IO APIC 24 pins
/// 56      Local APIC Timer<br>
/// 57      Local APIC LINT0<br>
/// 58      Local APIC LINT1<br>
/// 59      Local APIC Error<br>
/// 255     Local APIC Spurious-Interrupt (handler must do nothing (and even don't send an EOI))
pub fn general_interrupt_handler(
    interrupt_stack_frame: InterruptStackFrame,
    index: u8,
    error_code: Option<u64>,
) {
    match index {
        index if CPU_EXCEPTIONS_IDT_VECTORS_RANGE.contains(&index) => {
            // CPU Exception
            let exception =
                ExceptionVector::try_from(index).expect("Invalid exception vector number");

            match exception {
                ExceptionVector::Page => {
                    let cr2_virtual_address =
                        x86_64::registers::control::Cr2::read().expect("Invalid address in CR2");
                    panic!(
                        "Exception: {exception:?}\n\
                        Error code: {error_code:#?}\n\
                        CR2: 0x{cr2_virtual_address:X}
                        {interrupt_stack_frame:#?}"
                    );
                }
                _ => {
                    panic!(
                        "Exception: {exception:?}\n\
                        Error code: {error_code:#?}\n\
                        {interrupt_stack_frame:#?}"
                    );
                }
            }
        }
        index if IO_APIC_24_VECTORS_RANGE.contains(&index) => {
            if IO_APIC_ISA_IRQ_VECTORS_RANGE.contains(&index) {
                // PIT interrupt
                if index == 32 {
                    timers::pit::tick_interrupt_handler();
                } else {
                    crate::serial_println_lock_free!("IO APIC ISA IRQ interrupt: {index}");
                }
            } else {
                crate::serial_println_lock_free!("IO APIC *NOT* ISA IRQ interrupt: {index}");
            }
            apic::send_eoi();
        }
        LOCAL_APIC_TIMER_IDT_VECTOR => {
            crate::serial_println_lock_free!("LOCAL APIC TIMER interrupt");
            apic::send_eoi();
        }
        LOCAL_APIC_LINT0_IDT_VECTOR => {
            crate::serial_println_lock_free!("LOCAL APIC LINT0 interrupt");
            apic::send_eoi();
        }
        LOCAL_APIC_LINT1_IDT_VECTOR => {
            crate::serial_println_lock_free!("LOCAL APIC LINT1 interrupt");
            apic::send_eoi();
        }
        LOCAL_APIC_ERROR_IDT_VECTOR => {
            panic!("LOCAL APIC ERROR interrupt");
            apic::send_eoi();
        }
        LOCAL_APIC_SPURIOUS_IDT_VECTOR => {
            crate::serial_println_lock_free!("LOCAL APIC SPURIOUS interrupt");
            return;
        }
        _ => {
            unreachable!("Unexpected interrupt number: {index}!");
        }
    }
}
