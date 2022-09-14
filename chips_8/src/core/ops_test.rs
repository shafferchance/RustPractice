use super::*;

#[test]
fn initialized_properly() {
    let my_chip_8 = MyChips8::new();

    assert_eq!(0x200, my_chip_8.pc);
    assert_eq!(0x0, my_chip_8.opcode);
    assert_eq!(0x0, my_chip_8.i);
    assert_eq!(0x0, my_chip_8.sp);
}

#[test]
fn opcode_resolution() {
    let mut my_chip_8 = MyChips8::new();

    // Add some fake display data
    my_chip_8.gfx[0] = 1;

    assert_eq!(1, my_chip_8.gfx[0]);

    // Since this is at init PC is at 0x200
    my_chip_8.memory[0x200] = 0x0;  // CLS
    my_chip_8.memory[0x201] = 0xE0;  // no-op for clear
    my_chip_8.enumlate_cycle();
    assert_eq!(0, my_chip_8.gfx[0]);
    assert_eq!(0x202, my_chip_8.pc);
}

#[test]
fn a_opcode() {
    let mut my_chip_8 = MyChips8::new();

    // Load program into Index Register (I)
    my_chip_8.memory[0x200] = 0xA2;
    my_chip_8.memory[0x201] = 0x2A;

    my_chip_8.enumlate_cycle();
    assert_eq!(0x22A, my_chip_8.i);
}