use embedded_hal::{digital::OutputPin, spi::SpiBus};

use crate::{
    Ad983x, BitFlags, Config, DataFormat, Error, FrequencyRegister, PhaseRegister, PoweredDown,
};
use core::marker::PhantomData;

impl Config {
    pub(crate) fn with_high(self, mask: u16) -> Self {
        Config {
            bits: self.bits | mask,
        }
    }
    pub(crate) fn with_low(self, mask: u16) -> Self {
        Config {
            bits: self.bits & !mask,
        }
    }
}

impl BitFlags {
    pub(crate) const D15: u16 = 1 << 15;
    pub(crate) const D14: u16 = 1 << 14;
    pub(crate) const D13: u16 = 1 << 13;
    pub(crate) const B28: u16 = 1 << 13;
    pub(crate) const HLB: u16 = 1 << 12;
    pub(crate) const FSELECT: u16 = 1 << 11;
    pub(crate) const PSELECT: u16 = 1 << 10;
    pub(crate) const PIN_SW: u16 = 1 << 9;
    pub(crate) const RESET: u16 = 1 << 8;
    pub(crate) const SLEEP_MCLK: u16 = 1 << 7; // SLEEP1
    pub(crate) const SLEEP_DAC: u16 = 1 << 6; // SLEEP12
    pub(crate) const OPBITEN: u16 = 1 << 5;
    pub(crate) const SIGN_PIB: u16 = 1 << 4;
    pub(crate) const DIV2: u16 = 1 << 3;
    pub(crate) const MODE: u16 = 1 << 1;
}

impl<DEV, CS, IC> Ad983x<DEV, CS, IC> {
    pub(crate) fn create(spi: DEV, cs: CS) -> Self {
        Ad983x {
            spi,
            cs,
            control: Config {
                bits: BitFlags::RESET,
            },
            _ic: PhantomData,
        }
    }

    /// Destroy driver instance, return SPI bus instance and CS output pin.
    pub fn destroy(self) -> (DEV, CS) {
        (self.spi, self.cs)
    }
}

impl<DEV, CS, IC, E> Ad983x<DEV, CS, IC>
where
    DEV: SpiBus<Error = E>,
    CS: OutputPin<Error = E>,
{
    /// Resets the internal registers and leaves the device disabled.
    ///
    /// Note that this is ignored in AD9834/AD9838 devices if hardware pin
    /// control source is selected.
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        self.disable()
    }

    /// Disable the device (enable reset)
    ///
    /// This resets the internal registers.
    /// Note that this is ignored in AD9834/AD9838 devices if hardware pin
    /// control source is selected.
    pub fn disable(&mut self) -> Result<(), Error<E>> {
        let control = self.control.with_high(BitFlags::RESET);
        self.write_control(control)
    }

    /// Enable the device (disable reset)
    ///
    /// Note that this is ignored in AD9834/AD9838 devices if hardware pin
    /// control source is selected.
    pub fn enable(&mut self) -> Result<(), Error<E>> {
        let control = self.control.with_low(BitFlags::RESET);
        self.write_control(control)
    }

    fn check_value_fits<T>(value: T, bit_count: T) -> Result<(), Error<E>>
    where
        T: From<u8> + PartialOrd + core::ops::Shl<Output = T>,
    {
        if value >= (T::from(1) << bit_count) {
            Err(Error::InvalidArgument)
        } else {
            Ok(())
        }
    }

    /// Set the frequency as a 28-bit word
    ///
    /// This will change the mode to 28-bit if it is not used.
    /// Returns `Error::InvalidArgument` if providing a value that does not fit in 28 bits.
    pub fn set_frequency(
        &mut self,
        register: FrequencyRegister,
        value: u32,
    ) -> Result<(), Error<E>> {
        Self::check_value_fits(value, 28)?;
        let control = self.control.with_high(BitFlags::B28);
        self.write_control_if_different(control)?;
        let reg = Self::get_freq_register_bits(register);
        self.write_data(reg, value)
    }

    fn get_freq_register_bits(register: FrequencyRegister) -> u16 {
        match register {
            FrequencyRegister::F0 => BitFlags::D14,
            FrequencyRegister::F1 => BitFlags::D15,
        }
    }

    /// Set the frequency 14-bit MSBs
    ///
    /// This will deactivate the 28-bit mode if it is not already the case.
    /// Returns `Error::InvalidArgument` if providing a value that does not fit in 14 bits.
    pub fn set_frequency_msb(
        &mut self,
        register: FrequencyRegister,
        value: u16,
    ) -> Result<(), Error<E>> {
        Self::check_value_fits(value, 14)?;
        let control = self
            .control
            .with_low(BitFlags::B28)
            .with_high(BitFlags::HLB);
        self.write_control_if_different(control)?;
        let reg = Self::get_freq_register_bits(register);
        self.write(DataFormat::U16(reg | value))
    }

    /// Set the frequency 14-bit LSBs
    ///
    /// This will deactivate the 28-bit mode if it is not already the case.
    /// Returns `Error::InvalidArgument` if providing a value that does not fit in 14 bits.
    pub fn set_frequency_lsb(
        &mut self,
        register: FrequencyRegister,
        value: u16,
    ) -> Result<(), Error<E>> {
        Self::check_value_fits(value, 14)?;
        let control = self.control.with_low(BitFlags::B28).with_low(BitFlags::HLB);
        self.write_control_if_different(control)?;
        let reg = Self::get_freq_register_bits(register);
        self.write(DataFormat::U16(reg | value))
    }

    /// Select the frequency register that is used
    ///
    /// Note: this can be overriden through the FSELECT pin in AD9834/AD9838
    /// devices if hardware pin control source is selected.
    pub fn select_frequency(&mut self, register: FrequencyRegister) -> Result<(), Error<E>> {
        let control = match register {
            FrequencyRegister::F0 => self.control.with_low(BitFlags::FSELECT),
            FrequencyRegister::F1 => self.control.with_high(BitFlags::FSELECT),
        };
        self.write_control(control)
    }

    /// Set a phase register (12-bit value)
    ///
    /// Returns `Error::InvalidArgument` if providing a value that does not fit in 12 bits.
    pub fn set_phase(&mut self, register: PhaseRegister, value: u16) -> Result<(), Error<E>> {
        Self::check_value_fits(value, 12)?;
        let value = value | BitFlags::D14 | BitFlags::D15;
        let value = match register {
            PhaseRegister::P0 => value,
            PhaseRegister::P1 => value | BitFlags::D13,
        };
        self.write(DataFormat::U16(value))
    }

    /// Select the phase register that is used.
    ///
    /// Note: this can be overriden through the PSELECT pin in AD9834/AD9838
    /// devices if hardware pin control source is selected.
    pub fn select_phase(&mut self, register: PhaseRegister) -> Result<(), Error<E>> {
        let control = match register {
            PhaseRegister::P0 => self.control.with_low(BitFlags::PSELECT),
            PhaseRegister::P1 => self.control.with_high(BitFlags::PSELECT),
        };
        self.write_control(control)
    }

    /// Set device parts powered-down state.
    ///
    /// Note: This can be overriden through the SLEEP pin
    /// in AD9834/AD9838 devices if hardware pin control source is selected.
    pub fn set_powered_down(&mut self, config: PoweredDown) -> Result<(), Error<E>> {
        let control = match config {
            PoweredDown::Nothing => self
                .control
                .with_low(BitFlags::SLEEP_MCLK)
                .with_low(BitFlags::SLEEP_DAC),
            PoweredDown::Dac => self
                .control
                .with_low(BitFlags::SLEEP_MCLK)
                .with_high(BitFlags::SLEEP_DAC),
            PoweredDown::InternalClock => self
                .control
                .with_high(BitFlags::SLEEP_MCLK)
                .with_low(BitFlags::SLEEP_DAC),
            PoweredDown::DacAndInternalClock => self
                .control
                .with_high(BitFlags::SLEEP_MCLK)
                .with_high(BitFlags::SLEEP_DAC),
        };
        self.write_control(control)
    }

    pub(crate) fn write_control_if_different(&mut self, control: Config) -> Result<(), Error<E>> {
        if control != self.control {
            self.write_control(control)
        } else {
            Ok(())
        }
    }

    pub(crate) fn write_control(&mut self, control: Config) -> Result<(), Error<E>> {
        let payload = control.bits & 0b0011_1111_1111_1111;
        self.write(DataFormat::U16(payload))?;
        self.control = control;
        Ok(())
    }

    pub(crate) fn write(&mut self, payload: DataFormat) -> Result<(), Error<E>> {
        self.cs.set_low().map_err(Error::CSPinError)?;
        let error = match payload {
            DataFormat::U32(data) => {
                let d2s = data.to_be_bytes();
                self.spi.write(&d2s).map_err(Error::Spi)
            }
            DataFormat::U16(data) => {
                let d2s = data.to_be_bytes();
                self.spi.write(&d2s).map_err(Error::Spi)
            }
            DataFormat::U8(data) => {
                let d2s = data.to_be_bytes();
                self.spi.write(&d2s).map_err(Error::Spi)
            }
        };
        self.cs.set_high().map_err(Error::CSPinError)?;
        error
    }

    pub(crate) fn write_data(&mut self, reg: u16, payload: u32) -> Result<(), Error<E>> {
        self.cs.set_low().map_err(Error::CSPinError)?;
        let mut msb = ((payload & 0x0FFFC000) >> 14) as u16;
        msb = reg | msb;

        let mut lsb = (payload & 0x00003FFF) as u16;
        lsb = reg | lsb;
        self.spi.write(&lsb.to_be_bytes()).map_err(Error::Spi)?;
        self.spi.write(&msb.to_be_bytes()).map_err(Error::Spi)?;
        self.cs.set_high().map_err(Error::CSPinError)?;
        Ok(())
    }
}
