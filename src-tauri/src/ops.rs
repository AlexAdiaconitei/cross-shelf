//! Wishlist mutation state machine (spec 8.2, acceptance A13). Pure logic over a `Writer` trait,
//! so timeouts, 200-with-errors, 429 and cancellation are injected in tests. No real store is
//! touched here: real writers only exist once `docs/validation/<store>-write.md` does (spec 8.3).

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desired {
    Present,
    Absent,
}

/// A desired state, never a toggle: running it twice is harmless.
#[derive(Debug, Clone)]
pub struct Operation {
    pub id: String,
    pub product_id: String,
    pub desired: Desired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpStatus {
    Applied,
    AlreadyInDesiredState,
    Failed,
    NeedsReauth,
    NotSupported,
    Unverified,
    CancelledNotSent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendResult {
    /// The provider answered and its semantic result says success.
    Accepted,
    /// HTTP 200 (or similar) whose body carries errors.
    SemanticError,
    /// The request may or may not have been applied.
    Timeout,
    RateLimited(Duration),
    Unauthorized,
    NotSupported,
    Other,
}

pub trait Writer {
    fn read_state(&mut self, op: &Operation) -> Result<Desired, ()>;
    fn send(&mut self, op: &Operation) -> SendResult;
}

/// Local idempotency by `operation_id`: the same id never sends twice (double click, reconnect).
#[derive(Default)]
pub struct Ledger(HashMap<String, OpStatus>);

impl Ledger {
    pub fn get(&self, id: &str) -> Option<OpStatus> {
        self.0.get(id).copied()
    }
}

const MAX_RATE_LIMIT_WAITS: u32 = 3;

pub fn run(
    w: &mut dyn Writer,
    op: &Operation,
    ledger: &mut Ledger,
    cancelled: &dyn Fn() -> bool,
    wait: &mut dyn FnMut(Duration),
) -> OpStatus {
    if let Some(done) = ledger.get(&op.id) {
        return done;
    }
    let status = execute(w, op, cancelled, wait);
    // A cancelled-before-send operation may be started again; everything else is final.
    if status != OpStatus::CancelledNotSent {
        ledger.0.insert(op.id.clone(), status);
    }
    status
}

fn execute(
    w: &mut dyn Writer,
    op: &Operation,
    cancelled: &dyn Fn() -> bool,
    wait: &mut dyn FnMut(Duration),
) -> OpStatus {
    if cancelled() {
        return OpStatus::CancelledNotSent;
    }
    // Already where we want it: no request at all.
    if w.read_state(op) == Ok(op.desired) {
        return OpStatus::AlreadyInDesiredState;
    }
    let mut waits = 0;
    let mut retried_after_timeout = false;
    loop {
        match w.send(op) {
            SendResult::Accepted => return verify(w, op),
            SendResult::SemanticError | SendResult::Other => return OpStatus::Failed,
            SendResult::Unauthorized => return OpStatus::NeedsReauth,
            SendResult::NotSupported => return OpStatus::NotSupported,
            SendResult::RateLimited(after) => {
                waits += 1;
                if waits > MAX_RATE_LIMIT_WAITS {
                    return OpStatus::Failed;
                }
                wait(after);
                if cancelled() {
                    return OpStatus::CancelledNotSent;
                }
            }
            SendResult::Timeout => {
                // A timeout proves nothing: read first, retry at most once and only if unchanged.
                match w.read_state(op) {
                    Ok(s) if s == op.desired => return OpStatus::Applied,
                    Ok(_) if !retried_after_timeout => {
                        retried_after_timeout = true;
                        if cancelled() {
                            return OpStatus::CancelledNotSent;
                        }
                    }
                    _ => return OpStatus::Unverified,
                }
            }
        }
    }
}

/// Success needs the provider's answer *and* a later read that agrees.
fn verify(w: &mut dyn Writer, op: &Operation) -> OpStatus {
    match w.read_state(op) {
        Ok(s) if s == op.desired => OpStatus::Applied,
        _ => OpStatus::Unverified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// Scripted writer: queued send results; `applies_on_timeout` decides if a timed-out send landed.
    struct Fake {
        state: Desired,
        sends: VecDeque<SendResult>,
        sent: u32,
        applies_on_accept: bool,
        applies_on_timeout: bool,
        read_fails: bool,
    }

    impl Fake {
        fn new(state: Desired, sends: Vec<SendResult>) -> Self {
            Self {
                state,
                sends: sends.into(),
                sent: 0,
                applies_on_accept: true,
                applies_on_timeout: false,
                read_fails: false,
            }
        }
    }

    impl Writer for Fake {
        fn read_state(&mut self, _: &Operation) -> Result<Desired, ()> {
            if self.read_fails {
                Err(())
            } else {
                Ok(self.state)
            }
        }
        fn send(&mut self, op: &Operation) -> SendResult {
            self.sent += 1;
            let r = self.sends.pop_front().unwrap_or(SendResult::Other);
            let lands = match r {
                SendResult::Accepted => self.applies_on_accept,
                SendResult::Timeout => self.applies_on_timeout,
                _ => false,
            };
            if lands {
                self.state = op.desired;
            }
            r
        }
    }

    fn op(id: &str) -> Operation {
        Operation {
            id: id.into(),
            product_id: "p".into(),
            desired: Desired::Absent,
        }
    }

    fn go(w: &mut Fake, o: &Operation, ledger: &mut Ledger) -> OpStatus {
        run(w, o, ledger, &|| false, &mut |_| {})
    }

    #[test]
    fn success_requires_a_confirming_reread() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::Accepted]);
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::Applied
        );
    }

    #[test]
    fn accepted_but_reread_disagrees_is_unverified_not_applied() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::Accepted]);
        w.applies_on_accept = false;
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::Unverified
        );
    }

    #[test]
    fn already_in_desired_state_sends_nothing() {
        let mut w = Fake::new(Desired::Absent, vec![]);
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::AlreadyInDesiredState
        );
        assert_eq!(w.sent, 0);
    }

    #[test]
    fn timeout_that_landed_is_applied_without_a_retry() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::Timeout]);
        w.applies_on_timeout = true;
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::Applied
        );
        assert_eq!(
            w.sent, 1,
            "no blind retry after a timeout that actually applied"
        );
    }

    #[test]
    fn timeout_that_did_not_land_retries_once_after_reading() {
        let mut w = Fake::new(
            Desired::Present,
            vec![SendResult::Timeout, SendResult::Accepted],
        );
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::Applied
        );
        assert_eq!(w.sent, 2);
        let mut w = Fake::new(
            Desired::Present,
            vec![
                SendResult::Timeout,
                SendResult::Timeout,
                SendResult::Accepted,
            ],
        );
        assert_eq!(
            go(&mut w, &op("2"), &mut Ledger::default()),
            OpStatus::Unverified
        );
        assert_eq!(w.sent, 2, "never more than one retry");
    }

    #[test]
    fn timeout_with_unreadable_state_is_unverified() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::Timeout]);
        // First read (precondition) must succeed, later ones fail.
        w.read_fails = false;
        let o = op("1");
        let mut first = true;
        struct Flaky<'a>(&'a mut Fake, &'a mut bool);
        impl Writer for Flaky<'_> {
            fn read_state(&mut self, o: &Operation) -> Result<Desired, ()> {
                if *self.1 {
                    *self.1 = false;
                    self.0.read_state(o)
                } else {
                    Err(())
                }
            }
            fn send(&mut self, o: &Operation) -> SendResult {
                self.0.send(o)
            }
        }
        let mut f = Flaky(&mut w, &mut first);
        assert_eq!(
            run(&mut f, &o, &mut Ledger::default(), &|| false, &mut |_| {}),
            OpStatus::Unverified
        );
    }

    #[test]
    fn http_200_with_errors_is_a_failure() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::SemanticError]);
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::Failed
        );
    }

    #[test]
    fn rate_limit_waits_the_server_time_then_succeeds_or_gives_up() {
        let mut waited = vec![];
        let mut w = Fake::new(
            Desired::Present,
            vec![
                SendResult::RateLimited(Duration::from_secs(7)),
                SendResult::Accepted,
            ],
        );
        let s = run(
            &mut w,
            &op("1"),
            &mut Ledger::default(),
            &|| false,
            &mut |d| waited.push(d),
        );
        assert_eq!(
            (s, waited),
            (OpStatus::Applied, vec![Duration::from_secs(7)])
        );

        let r = SendResult::RateLimited(Duration::from_secs(1));
        let mut w = Fake::new(
            Desired::Present,
            vec![r.clone(), r.clone(), r.clone(), r.clone(), r],
        );
        assert_eq!(
            go(&mut w, &op("2"), &mut Ledger::default()),
            OpStatus::Failed
        );
        assert_eq!(w.sent, 4);
    }

    #[test]
    fn double_click_never_sends_twice() {
        let mut ledger = Ledger::default();
        let mut w = Fake::new(
            Desired::Present,
            vec![SendResult::Accepted, SendResult::Accepted],
        );
        let o = op("same");
        assert_eq!(go(&mut w, &o, &mut ledger), OpStatus::Applied);
        assert_eq!(go(&mut w, &o, &mut ledger), OpStatus::Applied);
        assert_eq!(w.sent, 1);
    }

    #[test]
    fn cancellation_stops_what_was_not_sent_and_can_restart() {
        let mut ledger = Ledger::default();
        let mut w = Fake::new(Desired::Present, vec![SendResult::Accepted]);
        let o = op("1");
        assert_eq!(
            run(&mut w, &o, &mut ledger, &|| true, &mut |_| {}),
            OpStatus::CancelledNotSent
        );
        assert_eq!(w.sent, 0);
        assert_eq!(go(&mut w, &o, &mut ledger), OpStatus::Applied);
    }

    #[test]
    fn auth_and_support_failures_are_reported_plainly() {
        let mut w = Fake::new(Desired::Present, vec![SendResult::Unauthorized]);
        assert_eq!(
            go(&mut w, &op("1"), &mut Ledger::default()),
            OpStatus::NeedsReauth
        );
        let mut w = Fake::new(Desired::Present, vec![SendResult::NotSupported]);
        assert_eq!(
            go(&mut w, &op("2"), &mut Ledger::default()),
            OpStatus::NotSupported
        );
    }

    #[test]
    fn partial_success_keeps_each_operation_separate() {
        let mut ledger = Ledger::default();
        let mut ok = Fake::new(Desired::Present, vec![SendResult::Accepted]);
        let mut bad = Fake::new(Desired::Present, vec![SendResult::SemanticError]);
        assert_eq!(go(&mut ok, &op("a"), &mut ledger), OpStatus::Applied);
        assert_eq!(go(&mut bad, &op("b"), &mut ledger), OpStatus::Failed);
        assert_eq!(ledger.get("a"), Some(OpStatus::Applied));
        assert_eq!(ledger.get("b"), Some(OpStatus::Failed));
    }
}
