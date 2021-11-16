use chrono::{
    naive::{NaiveDate, NaiveDateTime, NaiveTime},
    Datelike, Timelike,
};
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::{periph::rtc::RtcPeriph, reg};
use hal::{self, persist::PersistDatastore};

const RTCPRE: u32 = 8; // HSE / 8 = 1MHz
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

#[derive(Copy, Clone)]
struct WriteProtect(reg::rtc::Wpr<Crt>);

impl WriteProtect {
    fn disable(&self) {
        self.0.store(|r| r.write_key(0xCA));
        self.0.store(|r| r.write_key(0x53));
    }

    fn enable(&self) {
        self.0.store(|r| r.write_key(0xFF));
    }
}

pub struct BackupRegisters {
    backup: reg::rtc::Bkp0R<Srt>,
    write_protect: WriteProtect,
}

impl PersistDatastore for BackupRegisters {
    fn load<'a, T: From<&'a [u32]>>(&'a self) -> T {
        let array: &[u32; 20] = unsafe { core::mem::transmute(self.backup.as_ptr()) };
        T::from(&array[..])
    }

    fn save<T: AsRef<[u32]>>(&mut self, t: &T) {
        let slice = t.as_ref();
        let array: &mut [u32; 20] = unsafe { core::mem::transmute(self.backup.as_ptr()) };
        cortex_m::interrupt::free(|_cs| {
            self.write_protect.disable();
            array[..slice.len()].copy_from_slice(slice);
            self.write_protect.enable();
        })
    }
}

pub enum ClockSource {
    /// bypass
    LSE(bool),
    LSI,
    HSE,
}

pub struct RTC {
    tr: reg::rtc::Tr<Crt>,
    dr: reg::rtc::Dr<Crt>,
    isr: reg::rtc::Isr<Srt>,
    ssr: reg::rtc::Ssr<Crt>,
    prer: reg::rtc::Prer<Srt>,
    rcc_apb1enr_pwren: reg::rcc::apb1enr::Pwren<Srt>,
    rcc_cfgr_rtcpre: reg::rcc::cfgr::Rtcpre<Srt>,
    rcc_csr_lsi_rdy: reg::rcc::csr::Lsirdy<Srt>,
    rcc_csr_lsi_on: reg::rcc::csr::Lsion<Srt>,
    rcc_bdcr_bdrst: reg::rcc::bdcr::Bdrst<Srt>,
    rcc_bdcr_rtcen: reg::rcc::bdcr::Rtcen<Srt>,
    rcc_bdcr_rtcsel0: reg::rcc::bdcr::Rtcsel0<Srt>,
    rcc_bdcr_rtcsel1: reg::rcc::bdcr::Rtcsel1<Srt>,
    rcc_bdcr_lse_bypass: reg::rcc::bdcr::Lsebyp<Srt>,
    rcc_bdcr_lse_rdy: reg::rcc::bdcr::Lserdy<Srt>,
    rcc_bdcr_lse_on: reg::rcc::bdcr::Lseon<Srt>,
    pwr_cr_dbp: reg::pwr::cr::Dbp<Srt>,
    write_protect: WriteProtect,
}

impl RTC {
    fn init(&mut self, clock_source: ClockSource) {
        self.rcc_apb1enr_pwren.set_bit();
        self.pwr_cr_dbp.set_bit();

        self.rcc_bdcr_bdrst.set_bit();
        self.rcc_bdcr_bdrst.clear_bit();

        match clock_source {
            ClockSource::HSE => {
                self.rcc_cfgr_rtcpre.write_bits(RTCPRE);
                // FIXME: workaround: rtcsel0 and rtcsel1 cannot be written sperately
                unsafe { *(0x40023870 as *mut u32) |= 0x300 };
            }
            ClockSource::LSE(bypass) => {
                if bypass {
                    self.rcc_bdcr_lse_bypass.set_bit();
                }
                self.rcc_bdcr_lse_on.set_bit();
                while !self.rcc_bdcr_lse_rdy.read_bit() {}
                self.rcc_bdcr_rtcsel0.set_bit();
            }
            ClockSource::LSI => {
                self.rcc_csr_lsi_on.set_bit();
                while !self.rcc_csr_lsi_rdy.read_bit() {}
                self.rcc_bdcr_rtcsel1.set_bit();
            }
        }
        self.rcc_bdcr_rtcen.set_bit();

        self.write_protect.disable();
        self.enter_init();
        // 1MHz / 128 / 8192 = 1Hz
        self.prer.modify(|r| r.write_prediv_s(PREDIV_S));
        // NOTE: two sperate accesses must be performed
        self.prer.modify(|r| r.write_prediv_a(PREDIV_A));
        self.exit_init();
        self.write_protect.enable();
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
            self.write_protect.disable();
            self.enter_init();
            self._set_time(time);
            self.exit_init();
            self.write_protect.enable();
        })
    }

    fn set_date(&self, date: &NaiveDate) {
        cortex_m::interrupt::free(|_cs| {
            self.write_protect.disable();
            self.enter_init();
            self._set_date(date);
            self.exit_init();
            self.write_protect.enable();
        })
    }

    fn set_datetime(&self, datetime: &NaiveDateTime) {
        cortex_m::interrupt::free(|_cs| {
            self.write_protect.disable();
            self.enter_init();
            self._set_date(&datetime.date());
            self._set_time(&datetime.time());
            self.exit_init();
            self.write_protect.enable();
        })
    }
}

pub fn init(regs: RtcPeriph, clock_source: ClockSource) -> (RTC, BackupRegisters) {
    let write_protect = WriteProtect(regs.rtc_wpr.into_copy());
    let mut rtc = RTC {
        tr: regs.rtc_tr.into_copy(),
        dr: regs.rtc_dr.into_copy(),
        isr: regs.rtc_isr,
        ssr: regs.rtc_ssr.into_copy(),
        prer: regs.rtc_prer,
        rcc_apb1enr_pwren: regs.rcc_apb1enr_pwren,
        rcc_cfgr_rtcpre: regs.rcc_cfgr_rtcpre,
        rcc_csr_lsi_rdy: regs.rcc_csr_lsirdy,
        rcc_csr_lsi_on: regs.rcc_csr_lsion,
        rcc_bdcr_bdrst: regs.rcc_bdcr_bdrst,
        rcc_bdcr_rtcen: regs.rcc_bdcr_rtcen,
        rcc_bdcr_rtcsel0: regs.rcc_bdcr_rtcsel0,
        rcc_bdcr_rtcsel1: regs.rcc_bdcr_rtcsel1,
        rcc_bdcr_lse_bypass: regs.rcc_bdcr_lsebyp,
        rcc_bdcr_lse_rdy: regs.rcc_bdcr_lserdy,
        rcc_bdcr_lse_on: regs.rcc_bdcr_lseon,
        pwr_cr_dbp: regs.pwr_cr_dbp,
        write_protect,
    };
    rtc.init(clock_source);
    (rtc, BackupRegisters { backup: regs.rtc_bkp0r, write_protect })
}
