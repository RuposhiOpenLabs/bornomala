use crate::host_adapter::HostAdapter;
use bornomala_engine::{BornomalaEngine, EngineAction, InputScheme, KeyEvent, TextEdit};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Watchdog threshold: after 5 consecutive unhandled engine errors in a host,
/// Bornomala automatically enters fail-safe pass-through to protect the host process.
const WATCHDOG_MAX_FAILURES: usize = 5;

pub struct BornomalaTextService {
    engine: BornomalaEngine,
    failure_counter: AtomicUsize,
    is_disabled_by_watchdog: bool,
}

impl Default for BornomalaTextService {
    fn default() -> Self {
        Self::new()
    }
}

impl BornomalaTextService {
    pub fn new() -> Self {
        Self {
            engine: BornomalaEngine::new(InputScheme::ClassicUnicode),
            failure_counter: AtomicUsize::new(0),
            is_disabled_by_watchdog: false,
        }
    }

    pub fn set_scheme(&mut self, scheme: InputScheme) {
        self.engine.set_scheme(scheme);
    }

    pub fn active_scheme(&self) -> InputScheme {
        self.engine.active_scheme()
    }

    pub fn is_watchdog_tripped(&self) -> bool {
        self.is_disabled_by_watchdog
    }

    /// Process a physical key event and apply edits to the provided host adapter.
    /// Returns true if the key event was consumed by the text service.
    pub fn handle_key<H: HostAdapter>(&mut self, event: KeyEvent, host: &mut H) -> bool {
        // If watchdog tripped, fail-safe to pass-through
        if self.is_disabled_by_watchdog {
            return false;
        }

        let action = self.engine.process_key(&event);
        self.apply_action(action, host)
    }

    fn apply_action<H: HostAdapter>(&mut self, action: EngineAction, host: &mut H) -> bool {
        let mut consumed = action.consumed;

        for edit in action.edits {
            match edit {
                TextEdit::Insert(text) => {
                    host.insert_text(&text);
                    consumed = true;
                }
                TextEdit::ReplaceLast { count, text } => {
                    host.replace_last(count, &text);
                    consumed = true;
                }
                TextEdit::DeleteBack(count) => {
                    host.delete_back(count);
                    consumed = true;
                }
                TextEdit::PassThrough => {
                    // Do nothing on host; pass key down to application
                }
            }
        }

        if let Some(comp) = action.composition {
            host.set_composition(&comp);
            consumed = true;
        } else {
            host.clear_composition();
        }

        // Reset failure counter on successful key handling
        self.failure_counter.store(0, Ordering::Relaxed);
        consumed
    }

    /// Record an error into the watchdog; automatically disables engine if threshold exceeded.
    pub fn record_failure(&mut self, error_code: u32) {
        let failures = self.failure_counter.fetch_add(1, Ordering::SeqCst) + 1;
        // Bounded, content-free diagnostic logging (only error code and timestamp)
        eprintln!(
            "[Bornomala Diagnostic] Host event failure count: {} (error code: 0x{:X})",
            failures, error_code
        );

        if failures >= WATCHDOG_MAX_FAILURES {
            self.is_disabled_by_watchdog = true;
            eprintln!("[Bornomala Watchdog] Repeated failure threshold reached. Engine disabled for this host process.");
        }
    }
}
