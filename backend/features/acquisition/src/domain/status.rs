use crate::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEvent {
    Run,
    /// A Single was armed: it waits for a trigger from the start.
    ArmSingle,
    /// The Single captured its frame.
    SingleDone,
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
        (_, StatusEvent::ArmSingle) => Phase::WaitingForTrigger,
        (_, StatusEvent::Stop | StatusEvent::Lost) | (Phase::Stopped, _) => Phase::Stopped,
        (_, StatusEvent::SingleDone) => Phase::Done,
        (_, StatusEvent::Triggered) => Phase::Triggered,
        (_, StatusEvent::Auto) => Phase::Auto,
        (_, StatusEvent::TimedOut) => Phase::WaitingForTrigger,
    }
}
