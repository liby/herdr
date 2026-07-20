use std::collections::HashMap;

use super::{TerminalId, TerminalRuntime};

/// Server-owned live terminal runtimes, keyed by durable terminal id.
///
/// This sits outside `AppState` so pure state can stay focused on workspace,
/// pane, and terminal metadata while the server/application layer owns PTYs,
/// parser backends, detector tasks, and channels.
#[derive(Default)]
pub(crate) struct TerminalRuntimeRegistry {
    runtimes: HashMap<TerminalId, TerminalRuntime>,
    /// Test-only foreground cwd overrides. Real foreground cwds come from live
    /// process trees, which unit tests cannot construct.
    #[cfg(test)]
    test_foreground_cwds: HashMap<TerminalId, std::path::PathBuf>,
}

impl TerminalRuntimeRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get(&self, terminal_id: &TerminalId) -> Option<&TerminalRuntime> {
        self.runtimes.get(terminal_id)
    }

    pub(crate) fn foreground_cwd(&self, terminal_id: &TerminalId) -> Option<std::path::PathBuf> {
        #[cfg(test)]
        if let Some(cwd) = self.test_foreground_cwds.get(terminal_id) {
            return Some(cwd.clone());
        }
        self.get(terminal_id)
            .and_then(|runtime| runtime.foreground_cwd())
    }

    #[cfg(test)]
    pub(crate) fn set_test_foreground_cwd(
        &mut self,
        terminal_id: TerminalId,
        cwd: std::path::PathBuf,
    ) {
        self.test_foreground_cwds.insert(terminal_id, cwd);
    }

    pub(crate) fn insert(
        &mut self,
        terminal_id: TerminalId,
        runtime: TerminalRuntime,
    ) -> Option<TerminalRuntime> {
        self.runtimes.insert(terminal_id, runtime)
    }

    pub(crate) fn remove(&mut self, terminal_id: &TerminalId) -> Option<TerminalRuntime> {
        self.runtimes.remove(terminal_id)
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &TerminalRuntime> {
        self.runtimes.values()
    }

    #[cfg(unix)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&TerminalId, &TerminalRuntime)> {
        self.runtimes.iter()
    }

    #[cfg(unix)]
    pub(crate) fn set_handoff_readers_paused(&self, paused: bool) {
        for runtime in self.runtimes.values() {
            runtime.set_handoff_reader_paused(paused);
        }
    }

    #[cfg(unix)]
    pub(crate) fn assume_handoff_ownership(&mut self) {
        for runtime in self.runtimes.values_mut() {
            runtime.assume_handoff_ownership();
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.runtimes.len()
    }

    #[cfg(unix)]
    pub(crate) fn nudge_child_redraw_after_handoff(&self) {
        for runtime in self.runtimes.values() {
            runtime.nudge_child_redraw_after_handoff();
        }
    }

    #[cfg(unix)]
    pub(crate) fn drain_for_handoff(
        &mut self,
    ) -> impl Iterator<Item = (TerminalId, TerminalRuntime)> + '_ {
        self.runtimes.drain()
    }

    #[cfg(test)]
    pub(crate) fn drain(&mut self) -> impl Iterator<Item = (TerminalId, TerminalRuntime)> + '_ {
        self.runtimes.drain()
    }
}

impl From<HashMap<TerminalId, TerminalRuntime>> for TerminalRuntimeRegistry {
    fn from(runtimes: HashMap<TerminalId, TerminalRuntime>) -> Self {
        Self {
            runtimes,
            #[cfg(test)]
            test_foreground_cwds: HashMap::new(),
        }
    }
}
