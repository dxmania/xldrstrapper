use std::thread::sleep;
use std::time::{Duration, Instant};

use futures_lite::{future::{block_on, poll_once}, StreamExt};
use nusb::descriptors::TransferType;
use nusb::hotplug::HotplugEvent;
use nusb::transfer::{Buffer, Bulk, Direction, In, Out, TransferError};
use nusb::{DeviceInfo, Endpoint, MaybeFuture};

use crate::{DeviceId, Error, Result, Transport};

const WRITE_CHUNK: usize = 16 * 1024;

pub struct NusbTransport {
    ep_in: Endpoint<Bulk, In>,
    ep_out: Endpoint<Bulk, Out>,
}

fn io<E: std::fmt::Display>(ctx: &str) -> impl FnOnce(E) -> Error + '_ {
    move |e| Error::Io(format!("{ctx}: {e}"))
}

fn map_status(e: TransferError) -> Error {
    match e {
        TransferError::Cancelled => Error::Timeout,
        TransferError::Stall => Error::Stall,
        TransferError::Disconnected => Error::Disconnected,
        other => Error::Io(other.to_string()),
    }
}

impl NusbTransport {
    fn open(info: &DeviceInfo) -> Result<Self> {
        let dev = info.open().wait().map_err(io("open device"))?;
        let iface = dev
            .claim_interface(0)
            .wait()
            .map_err(io("claim interface 0 (on Windows: bind WinUSB with Zadig)"))?;

        let desc = iface
            .descriptor()
            .ok_or_else(|| Error::Protocol("no interface descriptor".into()))?;
        let (mut in_addr, mut out_addr) = (None, None);
        for ep in desc.endpoints() {
            if ep.transfer_type() == TransferType::Bulk {
                match ep.direction() {
                    Direction::In => in_addr = in_addr.or(Some(ep.address())),
                    Direction::Out => out_addr = out_addr.or(Some(ep.address())),
                }
            }
        }
        let (Some(i), Some(o)) = (in_addr, out_addr) else {
            return Err(Error::Protocol("need one bulk IN and one bulk OUT endpoint".into()));
        };

        Ok(Self {
            ep_in: iface.endpoint::<Bulk, In>(i).map_err(io("open bulk in"))?,
            ep_out: iface.endpoint::<Bulk, Out>(o).map_err(io("open bulk out"))?,
        })
    }
}

impl Transport for NusbTransport {
    fn read(&mut self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        let mps = self.ep_in.max_packet_size();
        let want = buf.len().div_ceil(mps) * mps;
        let c = self.ep_in.transfer_blocking(Buffer::new(want), timeout);
        c.status.map_err(map_status)?;
        let n = c.actual_len.min(buf.len());
        buf[..n].copy_from_slice(&c.buffer[..n]);
        Ok(n)
    }

    fn write(&mut self, data: &[u8], timeout: Duration) -> Result<()> {
        for chunk in data.chunks(WRITE_CHUNK) {
            let mut b = Buffer::new(chunk.len());
            b.extend_from_slice(chunk);
            let c = self.ep_out.transfer_blocking(b, timeout);
            c.status.map_err(map_status)?;
        }
        Ok(())
    }
}

fn matching(info: &DeviceInfo, ids: &[DeviceId]) -> Option<DeviceId> {
    ids.iter()
        .copied()
        .find(|id| id.vid == info.vendor_id() && id.pid == info.product_id())
}

pub fn wait_for_device(ids: &[DeviceId], timeout: Duration) -> Result<(DeviceId, NusbTransport)> {
    let mut watch = nusb::watch_devices().map_err(io("hotplug watch"))?;

    for info in nusb::list_devices().wait().map_err(io("list devices"))? {
        if let Some(id) = matching(&info, ids) {
            return Ok((id, NusbTransport::open(&info)?));
        }
    }

    let deadline = Instant::now() + timeout;
    loop {
        if let Some(Some(HotplugEvent::Connected(info))) = block_on(poll_once(watch.next())) {
            if let Some(id) = matching(&info, ids) {
                return Ok((id, NusbTransport::open(&info)?));
            }
            continue;
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout);
        }
        sleep(Duration::from_millis(2));
    }
}