use super::general_handler_func;
use x86_64::structures::idt::InterruptDescriptorTable;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Creates and loads IDT
pub fn init() {
    #[allow(static_mut_refs)]
    unsafe {
        x86_64::set_general_handler!(&mut IDT, general_handler_func);
        // Loads IDT using lidt
        IDT.load();
    }
}