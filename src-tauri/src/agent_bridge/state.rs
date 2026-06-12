use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Outcome {
    Answered(String),
    Dismissed,
    Timeout,
}

#[derive(Clone)]
pub struct BridgeState {
    pending: Arc<Mutex<HashMap<i64, Sender<Outcome>>>>,
    /// Serializes asks so the panel shows one question at a time.
    pub ask_serial: Arc<Mutex<()>>,
    /// Number of asks waiting for the serial lock (for the 429 cap).
    pub waiting: Arc<std::sync::atomic::AtomicUsize>,
}

pub struct PendingQuestion {
    id: i64,
    rx: Receiver<Outcome>,
    state: BridgeState,
}

impl BridgeState {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            ask_serial: Arc::new(Mutex::new(())),
            waiting: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn begin_question(&self, id: i64) -> PendingQuestion {
        let (tx, rx) = channel();
        self.pending.lock().unwrap().insert(id, tx);
        PendingQuestion {
            id,
            rx,
            state: self.clone(),
        }
    }

    /// Returns false if the id is unknown (already resolved / timed out).
    pub fn resolve(&self, id: i64, outcome: Outcome) -> bool {
        if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
            let _ = tx.send(outcome);
            true
        } else {
            false
        }
    }

    /// Id of the currently pending question, if any (used by GET /v1/pending
    /// and by the panel on mount).
    pub fn pending_id(&self) -> Option<i64> {
        self.pending.lock().unwrap().keys().next().copied()
    }
}

impl PendingQuestion {
    pub fn wait(self, timeout: Duration) -> Outcome {
        match self.rx.recv_timeout(timeout) {
            Ok(o) => o,
            Err(_) => {
                self.state.pending.lock().unwrap().remove(&self.id);
                Outcome::Timeout
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn answer_resolves_waiting_ask() {
        let st = BridgeState::new();
        let pending = st.begin_question(7);
        let st2 = st.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            assert!(st2.resolve(7, Outcome::Answered("ok".into())));
        });
        let out = pending.wait(Duration::from_secs(2));
        assert!(matches!(out, Outcome::Answered(a) if a == "ok"));
    }

    #[test]
    fn timeout_when_nobody_answers() {
        let st = BridgeState::new();
        let pending = st.begin_question(1);
        let out = pending.wait(Duration::from_millis(50));
        assert!(matches!(out, Outcome::Timeout));
        // resolved/cleaned up: late answer is rejected
        assert!(!st.resolve(1, Outcome::Answered("late".into())));
    }

    #[test]
    fn resolve_unknown_id_is_false() {
        let st = BridgeState::new();
        assert!(!st.resolve(99, Outcome::Dismissed));
    }
}
