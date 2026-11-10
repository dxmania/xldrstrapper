use std::collections::VecDeque;
use std::time::Duration;

use crate::{Error, Result, Transport};

#[derive(Debug, Clone)]
pub enum Step {
    DeviceSends(Vec<u8>),
    HostSends(Vec<u8>),
}

#[derive(Debug, Default)]
pub struct ReplayTransport {
    steps: VecDeque<Step>,
}

impl ReplayTransport {
    pub fn new(steps: impl IntoIterator<Item = Step>) -> Self {
        Self { steps: steps.into_iter().collect() }
    }

    pub fn finish(self) -> Result<()> {
        if self.steps.is_empty() {
            Ok(())
        } else {
            Err(Error::Replay(format!("{} scripted steps never happened", self.steps.len())))
        }
    }
}

impl Transport for ReplayTransport {
    fn read(&mut self, buf: &mut [u8], _timeout: Duration) -> Result<usize> {
        match self.steps.pop_front() {
            Some(Step::DeviceSends(d)) => {
                let n = d.len().min(buf.len());
                buf[..n].copy_from_slice(&d[..n]);
                Ok(n)
            }
            other => Err(Error::Replay(format!("host read, but trace had {other:?}"))),
        }
    }

    fn write(&mut self, data: &[u8], _timeout: Duration) -> Result<()> {
        match self.steps.pop_front() {
            Some(Step::HostSends(exp)) if exp == data => Ok(()),
            other => Err(Error::Replay(format!(
                "host wrote {} bytes, trace had {other:?}",
                data.len()
            ))),
        }
    }
}