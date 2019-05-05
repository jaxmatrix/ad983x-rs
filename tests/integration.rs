extern crate ad983x;
use ad983x::FrequencyRegister as FreqReg;
extern crate embedded_hal_mock as hal;
use self::hal::spi::Transaction as SpiTrans;

mod base;
use base::{destroy, new_ad9833, BitFlags as BF};

// TODO: wait for resolution of https://github.com/dbrgn/embedded-hal-mock/issues/25

#[test]
fn can_create_and_destroy() {
    let dev = new_ad9833(&[]);
    destroy(dev);
}

#[test]
fn can_enable() {
    let transitions = [SpiTrans::write(vec![0, 0])];
    let mut dev = new_ad9833(&transitions);
    dev.enable().unwrap();
    destroy(dev);
}

#[test]
fn can_disable() {
    let transitions = [SpiTrans::write(vec![BF::RESET, 0])];
    let mut dev = new_ad9833(&transitions);
    dev.disable().unwrap();
    destroy(dev);
}

#[test]
fn can_reset() {
    let transitions = [SpiTrans::write(vec![BF::RESET, 0])];
    let mut dev = new_ad9833(&transitions);
    dev.reset().unwrap();
    destroy(dev);
}

#[test]
fn cannot_set_too_fast_frequency() {
    let mut dev = new_ad9833(&[]);
    dev.set_frequency(FreqReg::F0, 1 << 28)
        .expect_err("Should return error");
    destroy(dev);
}

#[test]
fn can_set_freq0() {
    let transitions = [
        SpiTrans::write(vec![BF::B28 | BF::RESET, 0]),
        SpiTrans::write(vec![BF::FREQ0 | 0xD, 0xEF]),
        SpiTrans::write(vec![BF::FREQ0 | 0x26, 0xAF]),
    ];
    let mut dev = new_ad9833(&transitions);
    dev.set_frequency(FreqReg::F0, 0x9AB_CDEF).unwrap();
    destroy(dev);
}

#[test]
fn can_set_freq1() {
    let transitions = [
        SpiTrans::write(vec![BF::B28 | BF::RESET, 0]),
        SpiTrans::write(vec![BF::FREQ1 | 0xD, 0xEF]),
        SpiTrans::write(vec![BF::FREQ1 | 0x26, 0xAF]),
    ];
    let mut dev = new_ad9833(&transitions);
    dev.set_frequency(FreqReg::F1, 0x9AB_CDEF).unwrap();
    destroy(dev);
}

#[test]
fn can_select_freq0() {
    let transitions = [SpiTrans::write(vec![BF::RESET, 0])];
    let mut dev = new_ad9833(&transitions);
    dev.select_frequency(FreqReg::F0).unwrap();
    destroy(dev);
}

#[test]
fn can_select_freq1() {
    let transitions = [SpiTrans::write(vec![BF::FSELECT | BF::RESET, 0])];
    let mut dev = new_ad9833(&transitions);
    dev.select_frequency(FreqReg::F1).unwrap();
    destroy(dev);
}
