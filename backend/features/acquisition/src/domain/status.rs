use crate::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEvent {
    Run,
    Stop,
    Lost,
    Triggered,
    Auto,
    TimedOut,
}

/// A capture outcome that arrives while stopped does not revive the phase.
pub fn next_status(phase: Phase, event: StatusEvent) -> Phase {
    match (phase, event) {
        (_, StatusEvent::Run) => Phase::Armed,
        (_, StatusEvent::Stop | StatusEvent::Lost) | (Phase::Stopped, _) => Phase::Stopped,
        (_, StatusEvent::Triggered) => Phase::Triggered,
        (_, StatusEvent::Auto) => Phase::Auto,
        (_, StatusEvent::TimedOut) => Phase::WaitingForTrigger,
    }
}
