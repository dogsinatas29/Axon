pub mod win32_topology_strike;
pub mod gtk2_gobject_hell;
pub mod gtk_topology_strike;
pub mod c_topology_strike;

use crate::intelligence::common::types::CorpusFingerprint;
use super::trace_layering::TraceLayering;

pub trait PlatformStrikeSim {
    fn name(&self) -> &'static str;
    fn run_strike(&self, fingerprint: &CorpusFingerprint, seed: u64) -> Result<TraceLayering, String>;
}
