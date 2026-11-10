use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId {
    pub vid: u16,
    pub pid: u16,
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vid, self.pid)
    }
}

pub const OMAP4_ROM_IDS: [DeviceId; 2] = [
    DeviceId { vid: 0x0451, pid: 0xD00F },
    DeviceId { vid: 0x0451, pid: 0xD010 },
];

pub const KNOWN_IDS: &[DeviceId] = &OMAP4_ROM_IDS;