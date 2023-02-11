use std::{
    marker::PhantomData,
    mem::{size_of, zeroed},
};

use crate::{
    bindings::{
        CgosFanCount, CgosFanGetCurrent, CgosFanGetInfo, CgosFanSetLimits, CGOSFANINFO,
        CGOS_TEMP_BACKPLANE, CGOS_TEMP_BOARD, CGOS_TEMP_BOTDIMM_ENV, CGOS_TEMP_BOX,
        CGOS_TEMP_CHIPSETS, CGOS_TEMP_CPU, CGOS_TEMP_ENV, CGOS_TEMP_OTHER, CGOS_TEMP_TOPDIMM_ENV,
        CGOS_TEMP_VIDEO,
    },
    status::Status,
};

pub struct Fan<'library> {
    handle: u32,
    index: u32,
    _library_lifetime: PhantomData<&'library ()>,
}

impl<'library> Fan<'library> {
    pub(crate) fn amount(handle: u32) -> usize {
        unsafe { CgosFanCount(handle) as usize }
    }

    pub(crate) fn new(handle: u32, index: usize) -> Fan<'library> {
        Self {
            handle,
            index: index.try_into().unwrap(),
            _library_lifetime: PhantomData,
        }
    }

    pub fn info(&self) -> FanInfo {
        let mut info: CGOSFANINFO = unsafe { zeroed() };
        info.dwSize = size_of::<CGOSFANINFO>() as u32;
        assert_ne!(
            unsafe { CgosFanGetInfo(self.handle, self.index, &mut info) },
            0
        );
        info.into()
    }

    pub fn current(&self) -> (i32, Status) {
        let mut value = 0;
        let mut flags = 0;
        assert_ne!(
            unsafe { CgosFanGetCurrent(self.handle, self.index, &mut value, &mut flags) },
            0,
        );
        (value as i32, Status::from_bits_truncate(flags))
    }

    pub fn set_limits(&self, info: FanInfo) {
        let mut info = info.into();
        assert_ne!(
            unsafe { CgosFanSetLimits(self.handle, self.index, &mut info) },
            0,
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FanInfo {
    pub type_: FanType,
    pub status: Status,
    pub alarm: i32,
    pub speed_nominal: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub alarm_high: i32,
    pub hysteresis_high: i32,
    pub alarm_low: i32,
    pub hysteresis_low: i32,
    pub out_minimum: i32,
    pub out_maximum: i32,
}

impl From<CGOSFANINFO> for FanInfo {
    fn from(info: CGOSFANINFO) -> Self {
        Self {
            type_: info.dwType.into(),
            status: Status::from_bits_truncate(info.dwFlags),
            alarm: info.dwAlarm as i32,
            speed_nominal: info.dwSpeedNom as i32,
            minimum: info.dwMin as i32,
            maximum: info.dwMax as i32,
            alarm_high: info.dwAlarmHi as i32,
            hysteresis_high: info.dwHystHi as i32,
            alarm_low: info.dwAlarmLo as i32,
            hysteresis_low: info.dwHystLo as i32,
            out_minimum: info.dwOutMin as i32,
            out_maximum: info.dwOutMax as i32,
        }
    }
}

impl Into<CGOSFANINFO> for FanInfo {
    fn into(self) -> CGOSFANINFO {
        CGOSFANINFO {
            dwSize: size_of::<CGOSFANINFO>() as u32,
            dwType: self.type_.into(),
            dwFlags: self.status.bits(),
            dwAlarm: self.alarm as u32,
            dwSpeedNom: self.speed_nominal as u32,
            dwMin: self.minimum as u32,
            dwMax: self.maximum as u32,
            dwAlarmHi: self.alarm_high as u32,
            dwHystHi: self.hysteresis_high as u32,
            dwAlarmLo: self.alarm_low as u32,
            dwHystLo: self.hysteresis_low as u32,
            dwOutMin: self.out_minimum as u32,
            dwOutMax: self.out_maximum as u32,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FanType {
    Cpu,
    Box,
    Environment,
    Board,
    Backplane,
    Chipsets,
    Video,
    TopRAMEnvironment,
    BottomRAMEnvironment,
    Other,
}

impl Into<u32> for FanType {
    fn into(self) -> u32 {
        match self {
            FanType::Cpu => CGOS_TEMP_CPU,
            FanType::Box => CGOS_TEMP_BOX,
            FanType::Environment => CGOS_TEMP_ENV,
            FanType::Board => CGOS_TEMP_BOARD,
            FanType::Backplane => CGOS_TEMP_BACKPLANE,
            FanType::Chipsets => CGOS_TEMP_CHIPSETS,
            FanType::Video => CGOS_TEMP_VIDEO,
            FanType::Other => CGOS_TEMP_OTHER,
            FanType::TopRAMEnvironment => CGOS_TEMP_TOPDIMM_ENV,
            FanType::BottomRAMEnvironment => CGOS_TEMP_BOTDIMM_ENV,
        }
    }
}

impl From<u32> for FanType {
    fn from(value: u32) -> Self {
        match value {
            CGOS_TEMP_CPU => FanType::Cpu,
            CGOS_TEMP_BOX => FanType::Box,
            CGOS_TEMP_ENV => FanType::Environment,
            CGOS_TEMP_BOARD => FanType::Board,
            CGOS_TEMP_BACKPLANE => FanType::Backplane,
            CGOS_TEMP_CHIPSETS => FanType::Chipsets,
            CGOS_TEMP_VIDEO => FanType::Video,
            CGOS_TEMP_OTHER => FanType::Other,
            CGOS_TEMP_TOPDIMM_ENV => FanType::TopRAMEnvironment,
            CGOS_TEMP_BOTDIMM_ENV => FanType::BottomRAMEnvironment,
            _ => panic!("unexpected type {value}"),
        }
    }
}
