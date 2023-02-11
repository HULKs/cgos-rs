use bitflags::bitflags;

use crate::bindings::{
    CGOS_SENSOR_ACTIVE, CGOS_SENSOR_ALARM, CGOS_SENSOR_BROKEN, CGOS_SENSOR_SHORTCIRCUIT,
};

bitflags! {
    pub struct Status: u32 {
        const ACTIVE = CGOS_SENSOR_ACTIVE;
        const ALARM = CGOS_SENSOR_ALARM;
        const BROKEN = CGOS_SENSOR_BROKEN;
        const SHORT_CIRCUIT = CGOS_SENSOR_SHORTCIRCUIT;
    }
}
