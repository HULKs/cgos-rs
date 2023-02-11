use std::marker::PhantomData;

use bitflags::bitflags;

use crate::bindings::{
    CgosStorageAreaBlockSize, CgosStorageAreaCount, CgosStorageAreaErase,
    CgosStorageAreaEraseStatus, CgosStorageAreaIsLocked, CgosStorageAreaLock, CgosStorageAreaRead,
    CgosStorageAreaSize, CgosStorageAreaType, CgosStorageAreaUnlock, CgosStorageAreaWrite,
    CGOS_STORAGE_AREA_CMOS, CGOS_STORAGE_AREA_EEPROM, CGOS_STORAGE_AREA_FLASH,
    CGOS_STORAGE_AREA_RAM, CGOS_STORAGE_AREA_UNKNOWN,
};

pub struct StorageArea<'library> {
    handle: u32,
    unit: u32,
    _library_lifetime: PhantomData<&'library ()>,
}

impl<'library> StorageArea<'library> {
    pub(crate) fn amount(handle: u32, type_: StorageAreaType) -> usize {
        unsafe { CgosStorageAreaCount(handle, type_.bits()) as usize }
    }

    pub(crate) fn from_index(handle: u32, index: usize) -> StorageArea<'library> {
        Self {
            handle,
            unit: index.try_into().unwrap(),
            _library_lifetime: PhantomData,
        }
    }

    pub(crate) fn from_type(handle: u32, type_: StorageAreaType) -> StorageArea<'library> {
        Self {
            handle,
            unit: type_.bits(),
            _library_lifetime: PhantomData,
        }
    }

    pub fn type_(&self) -> StorageAreaType {
        StorageAreaType::from_bits_truncate(unsafe { CgosStorageAreaType(self.handle, self.unit) })
    }

    pub fn size(&self) -> usize {
        unsafe { CgosStorageAreaSize(self.handle, self.unit) as usize }
    }

    pub fn block_size(&self) -> usize {
        unsafe { CgosStorageAreaBlockSize(self.handle, self.unit) as usize }
    }

    pub fn read(&self, offset: usize, data: &mut [u8]) {
        assert_ne!(
            unsafe {
                CgosStorageAreaRead(
                    self.handle,
                    self.unit,
                    offset.try_into().unwrap(),
                    data.as_mut_ptr(),
                    data.len().try_into().unwrap(),
                )
            },
            0,
        );
    }

    pub fn write(&self, offset: usize, data: &[u8]) {
        assert_ne!(
            unsafe {
                CgosStorageAreaWrite(
                    self.handle,
                    self.unit,
                    offset.try_into().unwrap(),
                    data.as_ptr() as *mut _,
                    data.len().try_into().unwrap(),
                )
            },
            0,
        );
    }

    pub fn erase(&self, offset: usize, length: usize) {
        assert_ne!(
            unsafe {
                CgosStorageAreaErase(
                    self.handle,
                    self.unit,
                    offset.try_into().unwrap(),
                    length.try_into().unwrap(),
                )
            },
            0,
        );
    }

    pub fn erase_status(&self, offset: usize, length: usize) -> EraseStatus {
        let mut status = 0;
        assert_ne!(
            unsafe {
                CgosStorageAreaEraseStatus(
                    self.handle,
                    self.unit,
                    offset.try_into().unwrap(),
                    length.try_into().unwrap(),
                    &mut status,
                )
            },
            0,
        );
        status.into()
    }

    pub fn lock(&self, secret: &[u8]) {
        assert_ne!(
            unsafe {
                CgosStorageAreaLock(
                    self.handle,
                    self.unit,
                    0,
                    secret.as_ptr() as *mut _,
                    secret.len().try_into().unwrap(),
                )
            },
            0,
        );
    }

    pub fn unlock(&self, secret: &[u8]) {
        assert_ne!(
            unsafe {
                CgosStorageAreaUnlock(
                    self.handle,
                    self.unit,
                    0,
                    secret.as_ptr() as *mut _,
                    secret.len().try_into().unwrap(),
                )
            },
            0,
        );
    }

    pub fn is_locked(&self) -> bool {
        unsafe { CgosStorageAreaIsLocked(self.handle, self.unit, 0) != 0 }
    }
}

bitflags! {
    pub struct StorageAreaType: u32 {
        const UNKNOWN = CGOS_STORAGE_AREA_UNKNOWN;
        const EEPROM = CGOS_STORAGE_AREA_EEPROM;
        const FLASH = CGOS_STORAGE_AREA_FLASH;
        const CMOS = CGOS_STORAGE_AREA_CMOS;
        const RAM = CGOS_STORAGE_AREA_RAM;
    }
}

pub enum EraseStatus {
    Successful,
    InProgress,
    Failed,
}

impl From<u32> for EraseStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => EraseStatus::Successful,
            1 => EraseStatus::InProgress,
            2 => EraseStatus::Failed,
            _ => panic!("unexpected status {value}"),
        }
    }
}
