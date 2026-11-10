use crate::{rom_omap4::RomOmap4, DeviceId, Result, Transport, OMAP4_ROM_IDS};

#[derive(Debug, Clone)]
pub struct BootPlan {
    pub first_stage: Vec<u8>,
    pub image: Option<Vec<u8>>,
}

pub trait Backend {
    fn name(&self) -> &'static str;
    fn boot(&self, t: &mut dyn Transport, plan: &BootPlan) -> Result<()>;
}

pub fn backend_for(id: DeviceId) -> Option<Box<dyn Backend>> {
    if OMAP4_ROM_IDS.contains(&id) {
        return Some(Box::new(RomOmap4::default()));
    }
    None
}