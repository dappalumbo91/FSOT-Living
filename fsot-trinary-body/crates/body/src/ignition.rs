//! Zygote ignition graph — Linux init/systemd job, biological developmental cascade.
//!
//! Order is developmental, not arbitrary:
//!   serial nerve → tissue (heap) → germline → vagus → membrane → mind synapse → home LTM
//!
//! Each stage raises a Trit-ish readiness flag (latent → express).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceStage {
    Serial = 0,
    Heap = 1,
    Genome = 2,
    Intero = 3,
    Membrane = 4,
    MindAbi = 5,
    HomeVfs = 6,
    Scheduler = 7,
    Ready = 8,
}

impl ServiceStage {
    pub fn label(self) -> &'static str {
        match self {
            ServiceStage::Serial => "serial_nerve",
            ServiceStage::Heap => "tissue_heap",
            ServiceStage::Genome => "germline",
            ServiceStage::Intero => "vagus_intero",
            ServiceStage::Membrane => "organ_membrane",
            ServiceStage::MindAbi => "mind_synapse",
            ServiceStage::HomeVfs => "home_ltm",
            ServiceStage::Scheduler => "homeostasis_sched",
            ServiceStage::Ready => "soma_ready",
        }
    }

    pub fn biology(self) -> &'static str {
        match self {
            ServiceStage::Serial => "first sensory/motor axon online",
            ServiceStage::Heap => "metabolic substrate (ATP/tissue)",
            ServiceStage::Genome => "zygotic genome expression begins",
            ServiceStage::Intero => "interoceptive afferents (vagus)",
            ServiceStage::Membrane => "organ systems differentiate",
            ServiceStage::MindAbi => "cortico-brainstem synapse",
            ServiceStage::HomeVfs => "long-term memory stores mount",
            ServiceStage::Scheduler => "attention/metabolic priority live",
            ServiceStage::Ready => "organism viable for cortex attach",
        }
    }

    pub fn next(self) -> Option<ServiceStage> {
        match self {
            ServiceStage::Serial => Some(ServiceStage::Heap),
            ServiceStage::Heap => Some(ServiceStage::Genome),
            ServiceStage::Intero => Some(ServiceStage::Membrane),
            ServiceStage::Genome => Some(ServiceStage::Intero),
            ServiceStage::Membrane => Some(ServiceStage::MindAbi),
            ServiceStage::MindAbi => Some(ServiceStage::HomeVfs),
            ServiceStage::HomeVfs => Some(ServiceStage::Scheduler),
            ServiceStage::Scheduler => Some(ServiceStage::Ready),
            ServiceStage::Ready => None,
        }
    }
}

/// Developmental ignition state machine.
pub struct Ignition {
    pub stage: ServiceStage,
    pub completed: u8,
    pub log: Vec<String>,
}

impl Ignition {
    pub fn new() -> Self {
        Ignition {
            stage: ServiceStage::Serial,
            completed: 0,
            log: Vec::new(),
        }
    }

    /// Advance one stage; returns event line for outbox.
    pub fn advance(&mut self) -> Option<String> {
        if self.stage == ServiceStage::Ready {
            return None;
        }
        let done = self.stage;
        self.completed |= 1u8 << (done as u8);
        let line = format!(
            "BODY event=IGNITION stage={} bio={} ok=1",
            done.label(),
            done.biology()
        );
        self.log.push(line.clone());
        if let Some(n) = done.next() {
            self.stage = n;
        }
        Some(line)
    }

    /// Run full cascade at boot (host or kernel).
    pub fn run_boot_cascade(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        for _ in 0..16 {
            match self.advance() {
                Some(line) => out.push(line),
                None => break,
            }
            if self.stage == ServiceStage::Ready {
                if let Some(line) = self.advance() {
                    // Ready itself logs once when we enter it from Scheduler
                    let _ = line;
                }
                out.push(format!(
                    "BODY event=IGNITION stage={} bio={} ok=1",
                    ServiceStage::Ready.label(),
                    ServiceStage::Ready.biology()
                ));
                break;
            }
        }
        out
    }

    pub fn is_ready(&self) -> bool {
        self.stage == ServiceStage::Ready
    }

    pub fn status_line(&self) -> String {
        format!(
            "INIT stage={} ready={} mask={:#04x}",
            self.stage.label(),
            self.is_ready() as u8,
            self.completed
        )
    }
}

impl Default for Ignition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_reaches_ready() {
        let mut ig = Ignition::new();
        let lines = ig.run_boot_cascade();
        assert!(ig.is_ready());
        assert!(lines.iter().any(|l| l.contains("soma_ready") || l.contains("home_ltm")));
    }
}
