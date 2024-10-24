use embedded_hal::{digital::OutputPin, spi::SpiBus};

use crate::{marker, Ad983x, BitFlags, Error, OutputWaveform};

impl<DEV, CS, E> Ad983x<DEV, CS, marker::Ad9833Ad9837>
where
    DEV: SpiBus<Error = E>,
    CS: OutputPin<Error = E>,
{
    /// Create a new instance of an AD9833 device.
    ///
    /// Remember to call `reset()` before using the device after power up.
    pub fn new_ad9833(spi: DEV, cs: CS) -> Self {
        Self::create(spi, cs)
    }
    /// Create a new instance of an AD9837 device.
    ///
    /// Remember to call `reset()` before using the device after power up.
    pub fn new_ad9837(spi: DEV, cs: CS) -> Self {
        // Behaves the same as AD9833
        Self::create(spi, cs)
    }

    /// Set the output waveform
    pub fn set_output_waveform(&mut self, waveform: OutputWaveform) -> Result<(), Error<E>> {
        let control = match waveform {
            OutputWaveform::Sinusoidal => self
                .control
                .with_low(BitFlags::OPBITEN)
                .with_low(BitFlags::MODE),
            OutputWaveform::Triangle => self
                .control
                .with_low(BitFlags::OPBITEN)
                .with_high(BitFlags::MODE),
            OutputWaveform::SquareMsbOfDac => self
                .control
                .with_high(BitFlags::OPBITEN)
                .with_low(BitFlags::MODE)
                .with_high(BitFlags::DIV2),
            OutputWaveform::SquareMsbOfDacDiv2 => self
                .control
                .with_high(BitFlags::OPBITEN)
                .with_low(BitFlags::MODE)
                .with_low(BitFlags::DIV2),
        };
        self.write_control(control)
    }
}
