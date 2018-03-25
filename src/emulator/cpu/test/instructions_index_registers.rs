use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_ldx_immediate() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0xDE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0x00]);
    assert_eq!(cpu.x, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0xFF]);
    assert_eq!(cpu.x, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0024, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xA6, 0x24]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 3);
}

#[test]
fn test_ldx_zero_page_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xB6, 0x24]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldx_absolute() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xAE, 0xEF, 0xBE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldx_absolute_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xBE, 0xDF, 0xBE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_immediate() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0xDE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0x00]);
    assert_eq!(cpu.y, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0xFF]);
    assert_eq!(cpu.y, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0024, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xA4, 0x24]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 3);
}

#[test]
fn test_ldy_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xB4, 0x24]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_absolute() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xAC, 0xEF, 0xBE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_absolute_x() {
    let mut cpu = new_cpu();
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xBC, 0xDF, 0xBE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}
