use cortex_m::asm;
use embedded_hal::timer;

use core::marker::PhantomData;

use efm32;
use nb;

use void::Void;

pub struct RTC {
    _marked: PhantomData<()>,
}

pub mod rtc {
    use efm32;

    pub use efm32::cmu::lfapresc0::RTCW as Prescaler;

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum Ctrl {
        DebugRun = 2,
        Comp0Top = 4,
    }

    pub trait CtrlTrait {
        fn bits(self) -> u32;
    }

    impl CtrlTrait for [Ctrl; 0] {
        fn bits(self) -> u32 {
            0
        }
    }

    impl CtrlTrait for [Ctrl; 1] {
        fn bits(self) -> u32 {
            self[0] as u32
        }
    }

    impl CtrlTrait for [Ctrl; 2] {
        fn bits(self) -> u32 {
            self[0] as u32 | self[1] as u32
        }
    }

    pub struct Seconds(u32);

    impl Seconds {
        pub fn to_counter(self) -> u32 {
            let presc = unsafe { 1 << (*efm32::CMU::ptr()).lfapresc0.read().bits() };
            (32768 / presc) * self.0
        }
    }

    impl From<u32> for Seconds {
        fn from(c: u32) -> Self {
            Seconds(c)
        }
    }

}

impl RTC {
    pub fn new() -> Self {
        RTC {
            _marked: PhantomData,
        }
    }

    pub fn setup<CT: rtc::CtrlTrait>(&mut self, presc: rtc::Prescaler, ctrl: CT) {
        unsafe {
            (*efm32::CMU::ptr())
                .oscencmd
                .write(|w| w.lfrcoen().set_bit());
            (*efm32::CMU::ptr())
                .hfcoreclken0
                .modify(|_, w| w.le().set_bit());

            (*efm32::CMU::ptr())
                .lfapresc0
                .write(|w| w.rtc().variant(presc));
            (*efm32::CMU::ptr())
                .lfaclken0
                .modify(|_, w| w.rtc().set_bit());
            while (*efm32::CMU::ptr()).syncbusy.read().bits() & 3 > 0 {
                asm::nop();
            }

            (*efm32::RTC::ptr()).ctrl.write(|w| w.bits(ctrl.bits()));
        }
        Self::syncbusy();
    }

    pub fn default_setup(&mut self) {
        self.setup(rtc::Prescaler::DIV1, [rtc::Ctrl::Comp0Top]);
    }

    pub fn enable(&mut self) {
        unsafe { (*efm32::RTC::ptr()).ctrl.modify(|_, w| w.en().set_bit()) };
        Self::syncbusy();
    }

    pub fn disable(&mut self) {
        unsafe { (*efm32::RTC::ptr()).ctrl.modify(|_, w| w.en().clear_bit()) };
        Self::syncbusy();
    }

    pub fn get_counter(&self) -> u32 {
        unsafe { (*efm32::RTC::ptr()).cnt.read().bits() }
    }

    pub fn set_counter(&mut self, c: u32) {
        unsafe { (*efm32::RTC::ptr()).cnt.write(|w| w.cnt().bits(c)) };
    }

    pub fn get_comp0(&self) -> u32 {
        unsafe { (*efm32::RTC::ptr()).comp0.read().bits() }
    }

    pub fn set_comp0(&mut self, c: u32) {
        unsafe { (*efm32::RTC::ptr()).comp0.write(|w| w.comp0().bits(c)) };
        Self::syncbusy()
    }

    #[inline]
    fn syncbusy() {
        unsafe {
            while (*efm32::RTC::ptr()).syncbusy.read().ctrl().bit_is_set() {
                asm::nop();
            }
        }
    }
}

impl timer::CountDown for RTC {
    type Time = rtc::Seconds;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let ms_time: Self::Time = count.into();
        self.disable();
        self.set_comp0(ms_time.to_counter());
        self.enable();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.get_counter() >= self.get_comp0() - 1 {
            self.set_counter(0);
            return Ok(());
        }

        Err(nb::Error::WouldBlock)
    }
}
