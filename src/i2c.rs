use std::marker::PhantomData;

use crate::bindings::{
    CgosI2CCount, CgosI2CGetFrequency, CgosI2CGetMaxFrequency, CgosI2CIsAvailable, CgosI2CRead,
    CgosI2CReadRegister, CgosI2CSetFrequency, CgosI2CType, CgosI2CWrite, CgosI2CWriteReadCombined,
    CgosI2CWriteRegister, CGOS_I2C_TYPE_DDC, CGOS_I2C_TYPE_PRIMARY, CGOS_I2C_TYPE_SMB,
    CGOS_I2C_TYPE_UNKNOWN,
};

/// Error type for I2c operations
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum I2cErr {
    /// User-supplied index is out of range
    IdxOutOfRange,

    /// I2c bus transaction failed
    BusErr,
}

pub type I2cResult<T> = Result<T, I2cErr>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum I2cType {
    Unknown,
    Primary,
    Smb,
    Ddc,
    CongatecInternalUse(u32),
}

impl Into<u32> for I2cType {
    fn into(self) -> u32 {
        match self {
            Self::Unknown => CGOS_I2C_TYPE_UNKNOWN,
            Self::Primary => CGOS_I2C_TYPE_PRIMARY,
            Self::Smb => CGOS_I2C_TYPE_SMB,
            Self::Ddc => CGOS_I2C_TYPE_DDC,
            Self::CongatecInternalUse(x) => x,
        }
    }
}

impl From<u32> for I2cType {
    //note: On my devboard libcgos does return undeclared values for some busses, hence the need to return an error instead of panic
    fn from(value: u32) -> I2cType {
        match value {
            CGOS_I2C_TYPE_UNKNOWN => Self::Unknown,
            CGOS_I2C_TYPE_PRIMARY => Self::Primary,
            CGOS_I2C_TYPE_SMB => Self::Smb,
            CGOS_I2C_TYPE_DDC => Self::Ddc,
            _ => Self::CongatecInternalUse(value),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct I2c<'library> {
    handle: u32,
    index: u32,
    _library_lifetime: PhantomData<&'library ()>,
}
impl<'library> I2c<'library> {
    pub(crate) fn new(handle: u32, index: usize) -> I2cResult<Self> {
        let num_busses = Self::amount(handle);
        if index > num_busses.saturating_sub(1) {
            return Err(I2cErr::IdxOutOfRange);
        }

        let index = index.try_into().map_err(|_| I2cErr::IdxOutOfRange)?;
        let ret = Self {
            handle,
            index: index,
            _library_lifetime: PhantomData,
        };
        Ok(ret)
    }

    pub(crate) fn amount(handle: u32) -> usize {
        unsafe { CgosI2CCount(handle) as usize }
    }

    pub fn i2c_type(&'library self) -> I2cType {
        let raw = unsafe { CgosI2CType(self.handle, self.index) };
        I2cType::from(raw)
    }

    pub fn is_available(&'library self) -> bool {
        let raw = unsafe { CgosI2CIsAvailable(self.handle, self.index) };
        raw == 1
    }

    pub fn read(&'library self, bus_addr: u8, rd_data: &mut [u8]) -> I2cResult<()> {
        let retcode = unsafe {
            CgosI2CRead(
                self.handle,
                self.index,
                bus_addr,
                rd_data.as_mut_ptr(),
                rd_data.len() as u32,
            )
        };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(I2cErr::BusErr);
        }
    }

    pub fn write(&'library self, bus_addr: u8, wr_data: &[u8]) -> I2cResult<()> {
        let retcode = unsafe {
            dbg!(&wr_data, self.handle, self.index);
            CgosI2CWrite(
                self.handle,
                self.index,
                bus_addr,
                wr_data.as_ptr() as *mut u8,
                wr_data.len() as u32,
            )
        };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(I2cErr::BusErr);
        }
    }

    pub fn read_register(&'library self, bus_addr: u8, reg_addr: u16) -> I2cResult<u8> {
        let mut ret: u8 = 0;
        let retval = unsafe {
            CgosI2CReadRegister(
                self.handle,
                self.index,
                bus_addr,
                reg_addr,
                &mut ret as *mut u8,
            )
        };

        if retval != 0 {
            Ok(ret)
        } else {
            Err(I2cErr::BusErr)
        }
    }

    pub fn write_register(&'library self, bus_addr: u8, reg_addr: u16, val: u8) -> I2cResult<()> {
        let retval =
            unsafe { CgosI2CWriteRegister(self.handle, self.index, bus_addr, reg_addr, val) };

        if retval != 0 {
            Ok(())
        } else {
            Err(I2cErr::BusErr)
        }
    }

    pub fn write_read_combined(
        &'library self,
        bus_addr: u8,
        wr_data: &[u8],
        rd_data: &mut [u8],
    ) -> I2cResult<()> {
        let wr_len = wr_data.len();
        let retval = unsafe {
            CgosI2CWriteReadCombined(
                self.handle,
                self.index,
                bus_addr,
                wr_data.as_ptr() as *mut u8,
                wr_len.try_into().unwrap(),
                rd_data.as_mut_ptr(),
                rd_data.len() as u32,
            )
        };

        if retval != 0 {
            return Ok(());
        } else {
            return Err(I2cErr::BusErr);
        }
    }

    pub fn get_max_frequency(&'library self) -> I2cResult<u32> {
        let mut ret = 0;
        let retval =
            unsafe { CgosI2CGetMaxFrequency(self.handle, self.index, &mut ret as *mut u32) };

        if retval != 0 {
            return Ok(ret);
        } else {
            return Err(I2cErr::BusErr);
        }
    }

    pub fn get_frequency(&'library self) -> I2cResult<u32> {
        let mut ret = 0;
        let retval = unsafe { CgosI2CGetFrequency(self.handle, self.index, &mut ret as *mut u32) };

        if retval != 0 {
            return Ok(ret);
        } else {
            return Err(I2cErr::BusErr);
        }
    }

    pub fn set_frequency(&'library self, frequency: u32) -> I2cResult<()> {
        let retval = unsafe { CgosI2CSetFrequency(self.handle, self.index, frequency) };

        if retval != 0 {
            return Ok(());
        } else {
            return Err(I2cErr::BusErr);
        }
    }
}
