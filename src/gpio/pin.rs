use crate::gpio::mem::GpioMem;
use std::rc::Rc;
use crate::gpio::{Level, Mode};

// Maximum GPIO pins on the BCM2835. The actual number of pins
// exposed through the Pi's GPIO header depends on the model.
pub const MAX: usize = 54;

macro_rules! impl_pin {
    () => {
        /// Returns the GPIO pin number.
        ///
        /// Pins are addressed by their BCM numbers, rather than their physical location.
        #[inline]
        pub fn pin(&self) -> u8 {
            self.pin.pin
        }
    };
}


macro_rules! impl_output {
    () => {

        /// Sets the pin's output state to [`Low`].
        ///
        /// [`Low`]: enum.Level.html#variant.Low
        #[inline]
        pub fn set_low(&mut self) {
            self.pin.set_low()
        }

        /// Sets the pin's output state to [`High`].
        ///
        /// [`High`]: enum.Level.html#variant.High
        #[inline]
        pub fn set_high(&mut self) {
            self.pin.set_high()
        }

    };
}

macro_rules! impl_reset_on_drop {
    () => {
        /// Returns the value of `reset_on_drop`.
        pub fn reset_on_drop(&self) -> bool {
            self.reset_on_drop
        }

        /// When enabled, resets the pin's mode to its original state and disables the
        /// built-in pull-up/pull-down resistors when the pin goes out of scope.
        /// By default, this is set to `true`.
        ///
        /// ## Note
        ///
        /// Drop methods aren't called when a process is abnormally terminated, for
        /// instance when a user presses <kbd>Ctrl</kbd> + <kbd>C</kbd>, and the `SIGINT` signal
        /// isn't caught. You can catch those using crates such as [`simple_signal`].
        ///
        /// [`simple_signal`]: https://crates.io/crates/simple-signal
        pub fn set_reset_on_drop(&mut self, reset_on_drop: bool) {
            self.reset_on_drop = reset_on_drop;
        }
    };
}

macro_rules! impl_drop {
    ($struct:ident) => {
        impl Drop for $struct {
            /// Resets the pin's mode and disables the built-in pull-up/pull-down
            /// resistors if `reset_on_drop` is set to `true` (default).
            fn drop(&mut self) {
                if !self.reset_on_drop {
                    return;
                }

                if let Some(prev_mode) = self.prev_mode {
                    println!("resetting mode");
                    self.pin.set_mode(prev_mode);
                }
            }
        }
    };
}

macro_rules! impl_eq {
    ($struct:ident) => {
        impl PartialEq for $struct {
            fn eq(&self, other: &$struct) -> bool {
                self.pin == other.pin
            }
        }

        impl<'a> PartialEq<&'a $struct> for $struct {
            fn eq(&self, other: &&'a $struct) -> bool {
                self.pin == other.pin
            }
        }

        impl<'a> PartialEq<$struct> for &'a $struct {
            fn eq(&self, other: &$struct) -> bool {
                self.pin == other.pin
            }
        }

        impl Eq for $struct {}
    };
}

/// Unconfigured GPIO pin.
///
/// `Pin`s are constructed by retrieving them using [`Gpio::get`].
///
/// An unconfigured `Pin` can be used to read the pin's mode and logic level.
/// Converting the `Pin` to an [`InputPin`], [`OutputPin`] or [`IoPin`] through the
/// various `into_` methods available on `Pin` configures the appropriate mode, and
/// provides access to additional methods relevant to the selected pin mode.
///
///
/// [`digital::InputPin`]: ../../embedded_hal/digital/trait.InputPin.html
/// [`Gpio::get`]: struct.Gpio.html#method.get
/// [`InputPin`]: struct.InputPin.html
/// [`OutputPin`]: struct.OutputPin.html
/// [`IoPin`]: struct.IoPin.html
#[derive(Debug)]
pub struct Pin {
    pub(crate) pin: u8,
    gpio_mem: Rc<GpioMem>,
}



impl Pin {
    #[inline]
    pub(crate) fn new(pin: u8, gpio_mem: Rc<GpioMem>) -> Pin {
        Pin { pin, gpio_mem}
    }

    /// Returns the GPIO pin number.
    ///
    /// Pins are addressed by their BCM numbers, rather than their physical location.
    #[inline]
    pub fn pin(&self) -> u8 {
        self.pin
    }

    /// Returns the pin's mode.
    #[inline]
    pub fn mode(&self) -> Mode {
        self.gpio_mem.mode(self.pin)
    }

    /// Consumes the `Pin`, returns an [`OutputPin`] and sets its mode to [`Output`].
    ///
    /// [`OutputPin`]: struct.OutputPin.html
    /// [`Output`]: enum.Mode.html#variant.Output
    #[inline]
    pub fn into_output(self) -> OutputPin {
        OutputPin::new(self)
    }

    #[inline]
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.gpio_mem.set_mode(self.pin, mode);
    }

    #[inline]
    pub(crate) fn set_low(&mut self) {
        self.gpio_mem.set_low(self.pin);
    }

    #[inline]
    pub(crate) fn set_high(&mut self) {
        self.gpio_mem.set_high(self.pin);
    }

    #[inline]
    pub(crate) fn write(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        };
    }

}

impl_eq!(Pin);


/// GPIO pin configured as output.
///
/// `OutputPin`s are constructed by converting a [`Pin`] using [`Pin::into_output`].
/// The pin's mode is automatically set to [`Output`].
///
/// An `OutputPin` can be used to change a pin's output state.
///
/// The `embedded-hal` [`digital::OutputPin`] and [`PwmPin`] trait implementations for `OutputPin`
/// can be enabled by specifying the optional `hal` feature in the dependency
/// declaration for the `rppal` crate.
///
/// The `unproven` `embedded-hal` [`digital::InputPin`], [`digital::StatefulOutputPin`],
/// [`digital::ToggleableOutputPin`] and [`Pwm`] trait implementations for `OutputPin` can be enabled
/// by specifying the optional `hal-unproven` feature in the dependency declaration for
/// the `rppal` crate.
///
/// [`digital::InputPin`]: ../../embedded_hal/digital/trait.InputPin.html
/// [`digital::StatefulOutputPin`]: ../../embedded_hal/digital/trait.StatefulOutputPin.html
/// [`digital::ToggleableOutputPin`]: ../../embedded_hal/digital/trait.ToggleableOutputPin.html
/// [`Pwm`]: ../../embedded_hal/trait.Pwm.html
/// [`Pin`]: struct.Pin.html
/// [`Output`]: enum.Mode.html#variant.Output
/// [`Pin::into_output`]: struct.Pin.html#method.into_output
/// [`digital::OutputPin`]: ../../embedded_hal/digital/trait.OutputPin.html
/// [`PwmPin`]: ../../embedded_hal/trait.PwmPin.html
#[derive(Debug)]
pub struct OutputPin {
    pin: Pin,
    prev_mode: Option<Mode>,
    reset_on_drop: bool,
}

impl OutputPin {
    pub(crate) fn new(mut pin: Pin) -> OutputPin {
        let prev_mode = pin.mode();

        let prev_mode = if prev_mode == Mode::Output {
            None
        } else {
            pin.set_mode(Mode::Output);
            Some(prev_mode)
        };

        OutputPin {
            pin,
            prev_mode,
            reset_on_drop: true,
        }
    }

    impl_pin!();

    impl_output!();
    impl_reset_on_drop!();
}

impl_drop!(OutputPin);
impl_eq!(OutputPin);

