use std::{
    marker::PhantomData,
    mem::{size_of, zeroed},
};

use crate::{
    bindings::{
        CgosTemperatureCount, CgosTemperatureGetCurrent, CgosTemperatureGetInfo,
        CgosTemperatureSetLimits, CGOSTEMPERATUREINFO, CGOS_TEMP_BACKPLANE, CGOS_TEMP_BOARD,
        CGOS_TEMP_BOTDIMM_ENV, CGOS_TEMP_BOX, CGOS_TEMP_CHIPSETS, CGOS_TEMP_CPU, CGOS_TEMP_ENV,
        CGOS_TEMP_OTHER, CGOS_TEMP_TOPDIMM_ENV, CGOS_TEMP_VIDEO,
    },
    status::Status,
};

pub struct Temperature<'library> {
    handle: u32,
    index: u32,
    _library_lifetime: PhantomData<&'library ()>,
}

impl<'library> Temperature<'library> {
    pub(crate) fn amount(handle: u32) -> usize {
        unsafe { CgosTemperatureCount(handle) as usize }
    }

    pub(crate) fn new(handle: u32, index: usize) -> Temperature<'library> {
        Self {
            handle,
            index: index.try_into().unwrap(),
            _library_lifetime: PhantomData,
        }
    }

    pub fn info(&self) -> TemperatureInfo {
        let mut info: CGOSTEMPERATUREINFO = unsafe { zeroed() };
        info.dwSize = size_of::<CGOSTEMPERATUREINFO>() as u32;
        assert_ne!(
            unsafe { CgosTemperatureGetInfo(self.handle, self.index, &mut info) },
            0
        );
        info.into()
    }

    pub fn current(&self) -> (f32, Status) {
        let mut value = 0;
        let mut flags = 0;
        assert_ne!(
            unsafe { CgosTemperatureGetCurrent(self.handle, self.index, &mut value, &mut flags) },
            0,
        );
        (
            value as i32 as f32 / 1000.0,
            Status::from_bits_truncate(flags),
        )
    }

    pub fn set_limits(&self, info: TemperatureInfo) {
        let mut info = info.into();
        assert_ne!(
            unsafe { CgosTemperatureSetLimits(self.handle, self.index, &mut info) },
            0,
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TemperatureInfo {
    pub type_: TemperatureType,
    pub status: Status,
    pub alarm: u32,
    pub resolution: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub alarm_high: f32,
    pub hysteresis_high: f32,
    pub alarm_low: f32,
    pub hysteresis_low: f32,
}

impl From<CGOSTEMPERATUREINFO> for TemperatureInfo {
    fn from(info: CGOSTEMPERATUREINFO) -> Self {
        Self {
            type_: info.dwType.into(),
            status: Status::from_bits_truncate(info.dwFlags),
            alarm: info.dwAlarm,
            resolution: info.dwRes as i32 as f32 / 1000.0,
            minimum: info.dwMin as i32 as f32 / 1000.0,
            maximum: info.dwMax as i32 as f32 / 1000.0,
            alarm_high: info.dwAlarmHi as i32 as f32 / 1000.0,
            hysteresis_high: info.dwHystHi as i32 as f32 / 1000.0,
            alarm_low: info.dwAlarmLo as i32 as f32 / 1000.0,
            hysteresis_low: info.dwHystLo as i32 as f32 / 1000.0,
        }
    }
}

impl Into<CGOSTEMPERATUREINFO> for TemperatureInfo {
    fn into(self) -> CGOSTEMPERATUREINFO {
        CGOSTEMPERATUREINFO {
            dwSize: size_of::<CGOSTEMPERATUREINFO>() as u32,
            dwType: self.type_.into(),
            dwFlags: self.status.bits(),
            dwAlarm: self.alarm,
            dwRes: (self.resolution * 1000.0) as i32 as u32,
            dwMin: (self.minimum * 1000.0) as i32 as u32,
            dwMax: (self.maximum * 1000.0) as i32 as u32,
            dwAlarmHi: (self.alarm_high * 1000.0) as i32 as u32,
            dwHystHi: (self.hysteresis_high * 1000.0) as i32 as u32,
            dwAlarmLo: (self.alarm_low * 1000.0) as i32 as u32,
            dwHystLo: (self.hysteresis_low * 1000.0) as i32 as u32,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TemperatureType {
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

impl Into<u32> for TemperatureType {
    fn into(self) -> u32 {
        match self {
            TemperatureType::Cpu => CGOS_TEMP_CPU,
            TemperatureType::Box => CGOS_TEMP_BOX,
            TemperatureType::Environment => CGOS_TEMP_ENV,
            TemperatureType::Board => CGOS_TEMP_BOARD,
            TemperatureType::Backplane => CGOS_TEMP_BACKPLANE,
            TemperatureType::Chipsets => CGOS_TEMP_CHIPSETS,
            TemperatureType::Video => CGOS_TEMP_VIDEO,
            TemperatureType::Other => CGOS_TEMP_OTHER,
            TemperatureType::TopRAMEnvironment => CGOS_TEMP_TOPDIMM_ENV,
            TemperatureType::BottomRAMEnvironment => CGOS_TEMP_BOTDIMM_ENV,
        }
    }
}

impl From<u32> for TemperatureType {
    fn from(value: u32) -> Self {
        match value {
            CGOS_TEMP_CPU => TemperatureType::Cpu,
            CGOS_TEMP_BOX => TemperatureType::Box,
            CGOS_TEMP_ENV => TemperatureType::Environment,
            CGOS_TEMP_BOARD => TemperatureType::Board,
            CGOS_TEMP_BACKPLANE => TemperatureType::Backplane,
            CGOS_TEMP_CHIPSETS => TemperatureType::Chipsets,
            CGOS_TEMP_VIDEO => TemperatureType::Video,
            CGOS_TEMP_OTHER => TemperatureType::Other,
            CGOS_TEMP_TOPDIMM_ENV => TemperatureType::TopRAMEnvironment,
            CGOS_TEMP_BOTDIMM_ENV => TemperatureType::BottomRAMEnvironment,
            _ => panic!("unexpected type {value}"),
        }
    }
}
