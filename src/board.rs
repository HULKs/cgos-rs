use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::{size_of, zeroed},
    time::Duration,
};

use bitflags::bitflags;

use crate::{
    bindings::{
        CgosBoardClose, CgosBoardCount, CgosBoardGetBootCounter, CgosBoardGetInfoA,
        CgosBoardGetNameA, CgosBoardOpen, CgosBoardOpenByNameA, CGOSBOARDINFOA, CGOSTIME,
        CGOS_BOARD_CLASS_CPU, CGOS_BOARD_CLASS_IO, CGOS_BOARD_CLASS_VGA,
    },
    fan::Fan,
    storage_area::{StorageArea, StorageAreaType},
    temperature::Temperature,
};

pub const FLAGS: u32 = 0;

pub struct Board<'library> {
    handle: u32,
    _library_lifetime: PhantomData<&'library ()>,
}

impl<'library> Board<'library> {
    pub(crate) fn amount(class: BoardClass) -> usize {
        unsafe { CgosBoardCount(class.bits, FLAGS) as usize }
    }

    pub(crate) fn new(class: BoardClass, index: usize) -> Board<'library> {
        let mut handle = Default::default();
        assert_ne!(
            unsafe { CgosBoardOpen(class.bits, index.try_into().unwrap(), FLAGS, &mut handle) },
            0,
        );
        Self {
            handle,
            _library_lifetime: PhantomData,
        }
    }

    pub(crate) fn from_name(name: &str) -> Board<'library> {
        let name = CString::new(name).unwrap();
        let mut handle = Default::default();
        assert_ne!(
            unsafe { CgosBoardOpenByNameA(name.as_ptr(), &mut handle) },
            0,
        );
        Self {
            handle,
            _library_lifetime: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        const SIZE: usize = 128;
        let mut name = vec![0; SIZE];
        assert_ne!(
            unsafe { CgosBoardGetNameA(self.handle, name.as_mut_ptr() as *mut i8, SIZE as u32,) },
            0,
        );
        unsafe { CStr::from_ptr(name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn info(&self) -> BoardInfo {
        let mut info: CGOSBOARDINFOA = unsafe { zeroed() };
        info.dwSize = size_of::<CGOSBOARDINFOA>() as u32;
        assert_ne!(unsafe { CgosBoardGetInfoA(self.handle, &mut info) }, 0);
        info.into()
    }

    pub fn boot_count(&self) -> usize {
        let mut count = 0;
        assert_ne!(
            unsafe { CgosBoardGetBootCounter(self.handle, &mut count) },
            0,
        );
        count as usize
    }

    pub fn running_time(&self) -> Duration {
        let mut hours = 0;
        assert_ne!(
            unsafe { CgosBoardGetBootCounter(self.handle, &mut hours) },
            0,
        );
        Duration::from_secs(hours as u64 * 60 * 60)
    }

    pub fn get_number_of_temperatures(&self) -> usize {
        Temperature::amount(self.handle)
    }

    pub fn get_temperature(&'library self, index: usize) -> Temperature<'library> {
        Temperature::new(self.handle, index)
    }

    pub fn get_number_of_fans(&self) -> usize {
        Fan::amount(self.handle)
    }

    pub fn get_fan(&'library self, index: usize) -> Fan<'library> {
        Fan::new(self.handle, index)
    }

    pub fn get_number_of_storage_areas(&self, type_: StorageAreaType) -> usize {
        StorageArea::amount(self.handle, type_)
    }

    pub fn get_storage_area_from_index(&'library self, index: usize) -> StorageArea<'library> {
        StorageArea::from_index(self.handle, index)
    }

    pub fn get_storage_area_from_type(
        &'library self,
        type_: StorageAreaType,
    ) -> StorageArea<'library> {
        StorageArea::from_type(self.handle, type_)
    }
}

impl<'library> Drop for Board<'library> {
    fn drop(&mut self) {
        assert_ne!(unsafe { CgosBoardClose(self.handle) }, 0);
    }
}

bitflags! {
    pub struct BoardClass: u32 {
        const ALL = 0;
        const CPU = CGOS_BOARD_CLASS_CPU;
        const VGA = CGOS_BOARD_CLASS_VGA;
        const IO = CGOS_BOARD_CLASS_IO;
    }
}

#[derive(Clone, Debug)]
pub struct BoardInfo {
    pub board: String,
    pub board_sub: String,
    pub manufacturer: String,
    pub manufacturer_sub: u32,
    pub manufacturing_date: BoardTime,
    pub last_repair_date: BoardTime,
    pub serial_number: String,
    pub product_revision: String,
    pub system_bios_revision: u16,
    pub bios_interface_build_revision: u16,
    pub classes: BoardClass,
    pub primary_class: BoardClass,
    pub repair_counter: u32,
    pub part_number: String,
    pub european_article_number: String,
}

impl From<CGOSBOARDINFOA> for BoardInfo {
    fn from(info: CGOSBOARDINFOA) -> Self {
        let board = unsafe { CStr::from_ptr(info.szBoard.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        let board_sub = unsafe { CStr::from_ptr(info.szBoardSub.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        let manufacturer = unsafe { CStr::from_ptr(info.szManufacturer.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        let serial_number = unsafe { CStr::from_ptr(info.szSerialNumber.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        let product_revision = {
            let major = ((info.wProductRevision & 0xff00) >> 8) as u8 as char;
            let minor = (info.wProductRevision & 0xff) as u8 as char;
            format!("{major}.{minor}")
        };
        let part_number = unsafe { CStr::from_ptr(info.szPartNumber.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        let european_article_number = unsafe { CStr::from_ptr(info.szEAN.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string();
        Self {
            board,
            board_sub,
            manufacturer,
            manufacturer_sub: info.dwManufacturer,
            manufacturing_date: info.stManufacturingDate.into(),
            last_repair_date: info.stLastRepairDate.into(),
            serial_number,
            product_revision,
            system_bios_revision: info.wBiosInterfaceRevision,
            bios_interface_build_revision: info.wBiosInterfaceBuildRevision,
            classes: BoardClass::from_bits_truncate(info.dwClasses),
            primary_class: BoardClass::from_bits_truncate(info.dwPrimaryClass),
            repair_counter: info.dwRepairCounter,
            part_number,
            european_article_number,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BoardTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub millisecond: u16,
}

impl From<CGOSTIME> for BoardTime {
    fn from(time: CGOSTIME) -> Self {
        Self {
            year: time.wYear,
            month: time.wMonth,
            day_of_week: time.wDayOfWeek,
            day: time.wDay,
            hour: time.wHour,
            minute: time.wMinute,
            second: time.wSecond,
            millisecond: time.wMilliseconds,
        }
    }
}
