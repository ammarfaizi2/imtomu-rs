use efm32;

/// Disable Input/Output no pullup
pub struct Disabled;

/// Disable Input/Output with pullup
pub struct DisabledPullUp;

/// input MODE will be either:
/// - NoFilter   // no pull
/// - WithFilter // no pull
/// - PullDown   // no filter
/// - PullUp     // no filter
/// - PullDownWithFilter
/// - PullUpWithFilter
pub struct NoFilter;
pub struct WithFilter;
pub struct PullDown;
pub struct PullUp;
pub struct PullDownWithFilter;
pub struct PullupWithFilter;


/// output MODE will be either:
/// - PushPull
/// - PushPullDrive
/// - WiredOr         = OpenSource
/// - WiredOrPullDown = OpenSourcePullDown
/// - WiredAnd           = OpenDrain
/// - WiredAndWithFilter = OpenDrainWithFilter
/// - WiredAndPullUp     = OpenDrainPullUp
/// - WiredAndPullUpWithFilter = OpenDrainPullUpWithFilter
/// - WiredAndDrive      = OpenDrainDrive
/// - WiredAndDriveWithFilter  = OpenDrainDriveWithFilter
/// - WiredAndDrivePullUp      = OpenDrainDrivePullUp
/// - WiredAndDrivePullUpWithFilter = OpenDrainDrivePullUpWithFilter
pub struct PushPull;
pub struct PushPullDrive;

pub struct WiredOr;
pub type OpenSource = WiredOr;

pub struct WiredOrPullDown;
pub type OpenSourcePullDown = WiredOrPullDown;

pub struct WiredAnd;
pub type OpenDrain = WiredAnd;

pub struct WiredAndWithFilter;
pub type OpenDrainWithFilter = WiredAndWithFilter;

pub struct WiredAndPullUp;
pub type OpenDrainPullUp = WiredAndPullUp;

pub struct WiredAndPullUpWithFilter;
pub type OpenDrainPullUpWithFilter = WiredAndPullUpWithFilter;

pub struct WiredAndDrive;
pub type OpenDrainDrive = WiredAndDrive;

pub struct WiredAndDriveWithFilter;
pub type OpenDrainDriveWithFilter = WiredAndDriveWithFilter;

pub struct WiredAndDrivePullUp;
pub type OpenDrainDrivePullUp = WiredAndDrivePullUp;

pub struct WiredAndDrivePullUpWithFilter;
pub type OpenDrainDrivePullUpWithFilter = WiredAndDrivePullUpWithFilter;

pub struct GPIO;

impl GPIO {
    pub fn take(cmu: &mut efm32::CMU) -> Self {
        cmu.hfperclken0.modify(|_, w| w.gpio().bit(true));

        GPIO
    }

    pub fn split<MODE: GPIOPinSplitter>(&self) -> MODE::GPIOPin {
        MODE::split()
    }
}

pub trait GPIOPinSplitter {
    type GPIOPin;

    fn split() -> Self::GPIOPin;
}


macro_rules! gpio_pin_splitter {
    ($pin_struct:ident,
     $io_mode:ident,
     $modegroup:ident,
     $mode:ident,
     $setter:ident) => {
        impl GPIOPinSplitter for $pin_struct<$io_mode> {
            type GPIOPin = $pin_struct<$io_mode>;

            fn split() -> Self::GPIOPin {
                unsafe { (*efm32::GPIO::ptr()).$modegroup.modify(|_, w| w.$mode().$setter()) };
                $pin_struct { _m: PhantomData }
            }
        }
    };

    ($pin_struct:ident,
     $io_mode:ident,
     $modegroup:ident,
     $mode:ident,
     $setter:ident,
     $stmt:expr) => {
        impl GPIOPinSplitter for $pin_struct<$io_mode> {
            type GPIOPin = $pin_struct<$io_mode>;

            fn split() -> Self::GPIOPin {
                unsafe { (*efm32::GPIO::ptr()).$modegroup.modify(|_, w| w.$mode().$setter()) };
                $stmt;
                $pin_struct { _m: PhantomData }
            }
        }
    }
}

macro_rules! gpio_out_impl {
    ($pin_struct:ident,
     $io_mode:ident,
     $shift: expr,
     $outset:ident,
     $outclr:ident) => {
        impl<$io_mode> OutputPin for $pin_struct<$io_mode> {
            fn set_low(&mut self) {
                unsafe { (*efm32::GPIO::ptr()).$outclr.write(|w| w.bits(1 << $shift)) };
            }

            fn set_high(&mut self) {
                unsafe { (*efm32::GPIO::ptr()).$outset.write(|w| w.bits(1 << $shift)) };
            }
        }
    }
}

macro_rules! gpio {
    ($pin_struct:ident,
     $mode:ident,
     $shift:expr,
     $ctrl:ident,
     $modegroup:ident,
     $out:ident,
     $outset:ident,
     $outclr:ident,
     $outtgl:ident,
     $in:ident,
     $lock:ident) => {
        pub struct $pin_struct<Mode> {
            _m: PhantomData<Mode>,
        }

        // Disabled pin variants
        gpio_pin_splitter!($pin_struct, Disabled, $modegroup, $mode, disabled);
        gpio_pin_splitter!($pin_struct, DisabledPullUp, $modegroup, $mode, disabled,
                           unsafe { (*efm32::GPIO::ptr()).$outset.write(|w| w.bits(1 << $shift)) });

        // Output pin variants
        gpio_pin_splitter!($pin_struct, PushPull, $modegroup, $mode, pushpull);
        gpio_pin_splitter!($pin_struct, PushPullDrive, $modegroup, $mode, pushpulldrive);
        gpio_pin_splitter!($pin_struct, WiredOr, $modegroup, $mode, wiredor);
        gpio_pin_splitter!($pin_struct, WiredOrPullDown, $modegroup, $mode, wiredorpulldown);
        gpio_pin_splitter!($pin_struct, WiredAnd, $modegroup, $mode, wiredand);
        gpio_pin_splitter!($pin_struct, WiredAndWithFilter, $modegroup, $mode, wiredandfilter);
        gpio_pin_splitter!($pin_struct, WiredAndPullUp, $modegroup, $mode, wiredandpullup);
        gpio_pin_splitter!($pin_struct, WiredAndPullUpWithFilter, $modegroup, $mode, wiredandpullupfilter);
        gpio_pin_splitter!($pin_struct, WiredAndDrive, $modegroup, $mode, wiredanddrive);
        gpio_pin_splitter!($pin_struct, WiredAndDriveWithFilter, $modegroup, $mode, wiredanddrivefilter);
        gpio_pin_splitter!($pin_struct, WiredAndDrivePullUp, $modegroup, $mode, wiredanddrivepullup);
        gpio_pin_splitter!($pin_struct, WiredAndDrivePullUpWithFilter, $modegroup, $mode, wiredanddrivepullupfilter);

        gpio_out_impl!($pin_struct, PushPull, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, PushPullDrive, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredOr, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredOrPullDown, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAnd, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndWithFilter, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndPullUp, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndPullUpWithFilter, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndDrive, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndDriveWithFilter, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndDrivePullUp, $shift, $outset, $outclr);
        gpio_out_impl!($pin_struct, WiredAndDrivePullUpWithFilter, $shift, $outset, $outclr);
    }
}

pub mod pin {
    use super::*;
    use core::marker::PhantomData;
    use embedded_hal::digital::OutputPin;

    gpio!(A0, mode0, 0, pa_ctrl, pa_model, pa_dout, pa_doutset, pa_doutclr, pa_douttgl, pa_din, pa_pinlockn);
    gpio!(B7, mode7, 7, pb_ctrl, pb_model, pb_dout, pb_doutset, pb_doutclr, pb_douttgl, pb_din, pb_pinlockn);
    gpio!(B8, mode8, 0, pb_ctrl, pb_modeh, pb_dout, pb_doutset, pb_doutclr, pb_douttgl, pb_din, pb_pinlockn);
    gpio!(B11, mode11, 3, pb_ctrl, pb_modeh, pb_dout, pb_doutset, pb_doutclr, pb_douttgl, pb_din, pb_pinlockn);
    gpio!(B13, mode13, 5, pb_ctrl, pb_modeh, pb_dout, pb_doutset, pb_doutclr, pb_douttgl, pb_din, pb_pinlockn);
    gpio!(B14, mode14, 6, pb_ctrl, pb_modeh, pb_dout, pb_doutset, pb_doutclr, pb_douttgl, pb_din, pb_pinlockn);
    gpio!(C0, mode0, 0, pc_ctrl, pc_model, pc_dout, pc_doutset, pc_doutclr, pc_douttgl, pc_din, pc_pinlockn);
    gpio!(C1, mode1, 1, pc_ctrl, pc_model, pc_dout, pc_doutset, pc_doutclr, pc_douttgl, pc_din, pc_pinlockn);
    gpio!(C14, mode14, 6, pc_ctrl, pc_modeh, pc_dout, pc_doutset, pc_doutclr, pc_douttgl, pc_din, pc_pinlockn);
    gpio!(C15, mode15, 7, pc_ctrl, pc_modeh, pc_dout, pc_doutset, pc_doutclr, pc_douttgl, pc_din, pc_pinlockn);
    gpio!(E12, mode12, 4, pe_ctrl, pe_modeh, pe_dout, pe_doutset, pe_doutclr, pe_douttgl, pe_din, pe_pinlockn);
    gpio!(E13, mode13, 5, pe_ctrl, pe_modeh, pe_dout, pe_doutset, pe_doutclr, pe_douttgl, pe_din, pe_pinlockn);
    gpio!(F0, mode0, 0, pf_ctrl, pf_model, pf_dout, pf_doutset, pf_doutclr, pf_douttgl, pf_din, pf_pinlockn);
    gpio!(F1, mode1, 1, pf_ctrl, pf_model, pf_dout, pf_doutset, pf_doutclr, pf_douttgl, pf_din, pf_pinlockn);
    gpio!(F2, mode2, 2, pf_ctrl, pf_model, pf_dout, pf_doutset, pf_doutclr, pf_douttgl, pf_din, pf_pinlockn);
}
