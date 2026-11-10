use std::fmt;
use std::time::Duration;

use crate::{Backend, BootPlan, Result, Transport};

const MSG_BOOT: u32 = 0xF003_0002;
const MSG_GET_ID: u32 = 0xF003_0003;
const ASIC_ID_LEN: usize = 81;

pub struct AsicId(pub Vec<u8>);

impl fmt::Display for AsicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

pub struct RomOmap4 {
    pub timeout: Duration,
    pub settle: Duration,
}

impl Default for RomOmap4 {
    fn default() -> Self {
        Self { timeout: Duration::from_secs(5), settle: Duration::from_millis(250) }
    }
}

impl RomOmap4 {
    fn send_u32(&self, t: &mut dyn Transport, v: u32) -> Result<()> {
        t.write(&v.to_le_bytes(), self.timeout)
    }

    fn read_asic_id(&self, t: &mut dyn Transport) -> Result<AsicId> {
        self.send_u32(t, MSG_GET_ID)?;
        let mut buf = [0u8; 512];
        let n = t.read(&mut buf, self.timeout)?;
        if n != ASIC_ID_LEN {
            eprintln!("warning: ASIC ID is {n} bytes, expected {ASIC_ID_LEN}");
        }
        Ok(AsicId(buf[..n].to_vec()))
    }

    fn send_sized(&self, t: &mut dyn Transport, data: &[u8]) -> Result<()> {
        self.send_u32(t, data.len() as u32)?;
        t.write(data, self.timeout)
    }
}

impl Backend for RomOmap4 {
    fn name(&self) -> &'static str {
        "OMAP4 ROM peripheral boot"
    }

    fn boot(&self, t: &mut dyn Transport, plan: &BootPlan) -> Result<()> {
        let asic = self.read_asic_id(t)?;
        eprintln!("ASIC ID: {asic}");

        self.send_u32(t, MSG_BOOT)?;
        self.send_sized(t, &plan.first_stage)?;

        if let Some(image) = &plan.image {
            std::thread::sleep(self.settle);
            self.send_sized(t, image)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{ReplayTransport, Step};

    fn le(v: u32) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    fn quick() -> RomOmap4 {
        RomOmap4 { timeout: Duration::from_millis(10), settle: Duration::ZERO }
    }

    #[test]
    fn boots_two_stages_in_order() {
        let stage1 = vec![0xAA; 10];
        let image = vec![0xBB; 3];
        let mut t = ReplayTransport::new([
            Step::HostSends(le(MSG_GET_ID)),
            Step::DeviceSends(vec![0x01; ASIC_ID_LEN]),
            Step::HostSends(le(MSG_BOOT)),
            Step::HostSends(le(10)),
            Step::HostSends(stage1.clone()),
            Step::HostSends(le(3)),
            Step::HostSends(image.clone()),
        ]);
        quick()
            .boot(&mut t, &BootPlan { first_stage: stage1, image: Some(image) })
            .unwrap();
        t.finish().unwrap();
    }

    #[test]
    fn must_request_asic_id_before_anything_else() {
        let mut t = ReplayTransport::new([Step::DeviceSends(vec![0x01; ASIC_ID_LEN])]);
        let plan = BootPlan { first_stage: vec![0; 5], image: None };
        assert!(quick().boot(&mut t, &plan).is_err());
    }
}