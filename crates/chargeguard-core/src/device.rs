use crate::{error::Result, model::BatteryDevice};

pub trait PowerSupply {
    fn discover_devices(&self) -> Result<Vec<BatteryDevice>>;
    fn get_device(&self, id: &str) -> Result<BatteryDevice>;
    fn set_native_mode(&self, id: &str, mode: &str) -> Result<BatteryDevice>;
}
