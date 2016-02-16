use std::mem;

use super::TTYSettings;
use ::prelude::*;

fn default_settings() -> TTYSettings {
    TTYSettings {
        termios: unsafe { mem::uninitialized() }
    }
}

#[test]
#[cfg(target_os = "linux")]
fn tty_settings_sets_custom_baud_rate() {
    let mut settings = default_settings();

    settings.set_baud_rate(::BaudOther(250000)).unwrap();
    assert_eq!(settings.baud_rate(), Some(::BaudOther(250000)));
}

#[test]
fn tty_settings_sets_baud_rate() {
    let mut settings = default_settings();

    settings.set_baud_rate(::Baud600).unwrap();
    assert_eq!(settings.baud_rate(), Some(::Baud600));
}

#[test]
fn tty_settings_overwrites_baud_rate() {
    let mut settings = default_settings();

    settings.set_baud_rate(::Baud600).unwrap();
    settings.set_baud_rate(::Baud1200).unwrap();
    assert_eq!(settings.baud_rate(), Some(::Baud1200));
}

#[test]
fn tty_settings_sets_char_size() {
    let mut settings = default_settings();

    settings.set_char_size(::Bits8);
    assert_eq!(settings.char_size(), Some(::Bits8));
}

#[test]
fn tty_settings_overwrites_char_size() {
    let mut settings = default_settings();

    settings.set_char_size(::Bits8);
    settings.set_char_size(::Bits7);
    assert_eq!(settings.char_size(), Some(::Bits7));
}

#[test]
fn tty_settings_sets_parity_even() {
    let mut settings = default_settings();

    settings.set_parity(::ParityEven);
    assert_eq!(settings.parity(), Some(::ParityEven));
}

#[test]
fn tty_settings_sets_parity_odd() {
    let mut settings = default_settings();

    settings.set_parity(::ParityOdd);
    assert_eq!(settings.parity(), Some(::ParityOdd));
}

#[test]
fn tty_settings_sets_parity_none() {
    let mut settings = default_settings();

    settings.set_parity(::ParityEven);
    settings.set_parity(::ParityNone);
    assert_eq!(settings.parity(), Some(::ParityNone));
}

#[test]
fn tty_settings_sets_stop_bits_1() {
    let mut settings = default_settings();

    settings.set_stop_bits(::Stop2);
    settings.set_stop_bits(::Stop1);
    assert_eq!(settings.stop_bits(), Some(::Stop1));
}

#[test]
fn tty_settings_sets_stop_bits_2() {
    let mut settings = default_settings();

    settings.set_stop_bits(::Stop1);
    settings.set_stop_bits(::Stop2);
    assert_eq!(settings.stop_bits(), Some(::Stop2));
}

#[test]
fn tty_settings_sets_flow_control_software() {
    let mut settings = default_settings();

    settings.set_flow_control(::FlowSoftware);
    assert_eq!(settings.flow_control(), Some(::FlowSoftware));
}

#[test]
fn tty_settings_sets_flow_control_hardware() {
    let mut settings = default_settings();

    settings.set_flow_control(::FlowHardware);
    assert_eq!(settings.flow_control(), Some(::FlowHardware));
}

#[test]
fn tty_settings_sets_flow_control_none() {
    let mut settings = default_settings();

    settings.set_flow_control(::FlowHardware);
    settings.set_flow_control(::FlowNone);
    assert_eq!(settings.flow_control(), Some(::FlowNone));
}
