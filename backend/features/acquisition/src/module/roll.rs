//! Roll sessions (ADR 0017 items 1 and 3): which session is open, when it ends and what the items of it carry.
use sweepscope_kernel::contracts::{RollCaptured, RollCloseReason, RollContext, RollItem, RollPolled, RollPolledItem};

use super::effects::Output;
use crate::domain::RollKey;
use crate::Instrument;

struct Open {
    key: RollKey,
    context: RollContext,
}

/// A session is open while the instrument runs in roll mode. Any change of what it shows ends it and opens the next.
#[derive(Default)]
pub struct RollSessions {
    counter: u32,
    open: Option<Open>,
}

impl RollSessions {
    /// Opens, closes or replaces the session so it matches the instrument; call after every change of it.
    pub fn sync(&mut self, instrument: &Instrument, outputs: &mut Vec<Output>) {
        let wanted = instrument.ready() && instrument.running() && instrument.in_roll();
        let changed = self
            .open
            .as_ref()
            .is_some_and(|open| instrument.roll_key().as_ref() != Some(&open.key));
        match (self.open.is_some(), wanted) {
            (true, false) => self.close(close_reason(instrument), outputs),
            (true, true) if changed => {
                self.close(RollCloseReason::SetupChanged, outputs);
                self.open(instrument, outputs);
            }
            (false, true) => self.open(instrument, outputs),
            _ => {}
        }
    }

    /// The item of an open session; polls from before the last change of setup are the caller's to drop.
    pub fn item(&self, polled: RollPolled, instrument: &Instrument) -> Option<RollCaptured> {
        let open = self.open.as_ref()?;
        let item = match polled.item {
            RollPolledItem::Samples { first_index, frame } => RollItem::Samples {
                first_index,
                frame: instrument.correct_frame(&frame).frame,
            },
            RollPolledItem::Gap { after, missed } => RollItem::Gap { after, missed },
        };
        Some(RollCaptured {
            session: open.context.session,
            item,
            context: open.context,
        })
    }

    fn open(&mut self, instrument: &Instrument, outputs: &mut Vec<Output>) {
        let Some(key) = instrument.roll_key() else {
            return;
        };
        self.counter += 1;
        let context = instrument.roll_context(self.counter);
        outputs.push(roll_output(RollItem::Opened, context));
        self.open = Some(Open { key, context });
    }

    fn close(&mut self, reason: RollCloseReason, outputs: &mut Vec<Output>) {
        if let Some(open) = self.open.take() {
            outputs.push(roll_output(RollItem::Closed { reason }, open.context));
        }
    }
}

fn roll_output(item: RollItem, context: RollContext) -> Output {
    Output::Roll(RollCaptured {
        session: context.session,
        item,
        context,
    })
}

fn close_reason(instrument: &Instrument) -> RollCloseReason {
    if !instrument.ready() {
        RollCloseReason::Disconnected
    } else if !instrument.running() {
        RollCloseReason::Stopped
    } else {
        RollCloseReason::LeftRoll
    }
}
