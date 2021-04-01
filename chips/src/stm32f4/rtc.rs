use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{Datelike, Timelike};
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::rtc::RtcPeriph;
use drone_stm32_map::reg;
use hal::{self, persist::PersistDatastore};

const PREDIV_A: u32 = 0x7F;
const PREDIV_S: u32 = 0x1FFF;

struct BCD(u32);

impl BCD {
    fn tens(&self) -> u32 {
        self.0 / 10
    }

    fn units(&self) -> u32 {
        self.0 % 10
    }
}

impl From<(u32, u32)> for BCD {
    fn from(value: (u32, u32)) -> Self {
        Self((value.0 * 10 + value.1) as u32)
    }
}

impl Into<u8> for BCD {
    fn into(self) -> u8 {
        self.0 as u8
    }
}

#[derive(Copy, Clone)]
pub struct RTCReader {
    tr: reg::rtc::Tr<Crt>,
    dr: reg::rtc::Dr<Crt>,
    ssr: reg::rtc::Ssr<Crt>,
}

impl hal::rtc::RTCReader for RTCReader {
    fn date(&self) -> NaiveDate {
        let reg = self.dr.load();
        let year: BCD = (reg.yt(), reg.yu()).into();
        let month: BCD = (reg.mt() as u32, reg.mu()).into();
        let day: BCD = (reg.dt(), reg.du()).into();
        NaiveDate::from_ymd(year.0 as i32 + 1970, month.0, day.0)
    }

    fn time(&self) -> NaiveTime {
        let reg = self.tr.load();
        let hour: BCD = (reg.ht(), reg.hu()).into();
        let minute: BCD = (reg.mnt(), reg.mnu()).into();
        let second: BCD = (reg.st(), reg.su()).into();
        let sub_second = (PREDIV_S - self.ssr.load().ss()) / (PREDIV_S + 1);
        NaiveTime::from_hms_milli(hour.0, minute.0, second.0, sub_second)
    }
}

unsafe impl Send for RTCReader {}
unsafe impl Sync for RTCReader {}

pub struct BackupRegisters(reg::rtc::Bkp0R<Srt>);

impl PersistDatastore for BackupRegisters {
    fn load<'a, T: From<&'a [u32]>>(&'a self) -> T {
        let array: &[u32; 20] = unsafe { core::mem::transmute(self.0.as_ptr()) };
        T::from(&array[..])
    }

    fn save<T: AsRef<[u32]>>(&mut self, t: &T) {
        let slice = t.as_ref();
        let array: &mut [u32; 20] = unsafe { core::mem::transmute(self.0.as_ptr()) };
        array[..slice.len()].copy_from_slice(slice)
    }
}

pub struct RTC {
    tr: reg::rtc::Tr<Crt>,
    dr: reg::rtc::Dr<Crt>,
    wpr: reg::rtc::Wpr<Srt>,
    isr: reg::rtc::Isr<Srt>,
    ssr: reg::rtc::Ssr<Crt>,
}

impl RTC {
    fn init(&mut self, prer: reg::rtc::Prer<Srt>) {
        self.disable_write_protect();
        self.enter_init();
        prer.modify(|r| r.write_prediv_s(PREDIV_S)); // 1MHz / 128 / 8192 = 1Hz
        prer.modify(|r| r.write_prediv_a(PREDIV_A)); // NOTE: two sperate accesses must be performed
        self.exit_init();
        self.enable_write_protect();
    }

    pub fn new(regs: RtcPeriph) -> Self {
        let mut rtc = Self {
            tr: regs.rtc_tr.into_copy(),
            dr: regs.rtc_dr.into_copy(),
            wpr: regs.rtc_wpr,
            isr: regs.rtc_isr,
            ssr: regs.rtc_ssr.into_copy(),
        };
        rtc.init(regs.rtc_prer);
        rtc
    }

    pub fn disable_write_protect(&mut self) {
        self.wpr.store(|r| r.write_key(0xCA));
        self.wpr.store(|r| r.write_key(0x53));
    }

    pub fn enable_write_protect(&mut self) {
        self.wpr.store(|r| r.write_key(0xFF));
    }

    fn enter_init(&self) {
        self.isr.modify(|r| r.set_init());
        while !self.isr.initf.read_bit() {}
    }

    fn exit_init(&self) {
        self.isr.modify(|r| r.clear_init());
        while self.isr.initf.read_bit() {}
    }

    fn _set_time(&self, time: &NaiveTime) {
        let hour = BCD(time.hour());
        let minute = BCD(time.minute());
        let second = BCD(time.second());
        self.tr.modify(|r| {
            r.write_ht(hour.tens()).write_hu(hour.units());
            r.write_mnt(minute.tens()).write_mnu(minute.units());
            r.write_st(second.tens()).write_su(second.units())
        });
    }

    fn _set_date(&self, date: &NaiveDate) {
        let year = BCD(core::cmp::max(date.year() as u32, 1970) - 1970);
        let month = BCD(date.month());
        let day = BCD(date.day());
        self.dr.modify(|r| {
            r.write_yt(year.tens()).write_yu(year.units());
            if month.tens() > 0 { r.set_mt() } else { r.clear_mt() }.write_mu(month.units());
            r.write_dt(day.tens()).write_du(day.units());
            r.write_wdu(date.weekday().number_from_monday())
        });
    }

    pub fn reader(&self) -> RTCReader {
        RTCReader { tr: self.tr, dr: self.dr, ssr: self.ssr }
    }
}

impl hal::rtc::RTCWriter for RTC {
    fn set_time(&self, time: &NaiveTime) {
        cortex_m::interrupt::free(|_cs| {
            self.enter_init();
            self._set_time(time);
            self.exit_init();
        })
    }

    fn set_date(&self, date: &NaiveDate) {
        cortex_m::interrupt::free(|_cs| {
            self.enter_init();
            self._set_date(date);
            self.exit_init();
        })
    }

    fn set_datetime(&self, datetime: &NaiveDateTime) {
        cortex_m::interrupt::free(|_cs| {
            self.enter_init();
            self._set_date(&datetime.date());
            self._set_time(&datetime.time());
            self.exit_init();
        })
    }
}

pub fn init(regs: RtcPeriph) -> (RTC, BackupRegisters) {
    let mut rtc = RTC {
        tr: regs.rtc_tr.into_copy(),
        dr: regs.rtc_dr.into_copy(),
        wpr: regs.rtc_wpr,
        isr: regs.rtc_isr,
        ssr: regs.rtc_ssr.into_copy(),
    };
    rtc.init(regs.rtc_prer);
    (rtc, BackupRegisters(regs.rtc_bkp0r))
}
