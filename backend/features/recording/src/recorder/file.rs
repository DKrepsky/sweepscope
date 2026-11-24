//! The open `.ssroll` file: turns roll items into records, tracks segments and gaps, flushes on demand.
use std::io::{self, BufWriter, ErrorKind, Write};

use sweepscope_kernel::contracts::{Frame, RollContext, RollItem};

use crate::files::RecordWriter;
use crate::format::{self, EndReason, SegmentSetup};

const BUFFER_BYTES: usize = 64 * 1024;

/// The recording cannot go on; `text` is what the user reads, without any path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failed {
    pub text: &'static str,
}

impl Failed {
    pub const FILE_REMOVED: Self = Self {
        text: "the file was removed",
    };

    pub fn of(error: &io::Error) -> Self {
        let text = match error.kind() {
            ErrorKind::StorageFull | ErrorKind::QuotaExceeded => "the disk is full",
            ErrorKind::PermissionDenied => "permission denied",
            ErrorKind::NotFound => "the file was removed",
            _ => "the file cannot be written",
        };
        Self { text }
    }
}

pub enum Flow {
    Continue,
    Ended,
}

struct Segment {
    setup: SegmentSetup,
    expected_index: u64,
}

struct PendingGap {
    session: u32,
    after: u64,
    missed: u64,
}

pub struct RecordFile {
    writer: BufWriter<Box<dyn RecordWriter>>,
    segment: Option<Segment>,
    // Indices restart with a session, so a gap that arrives before the samples waits for its segment record.
    pending_gaps: Vec<PendingGap>,
    accepted_bytes: u64,
    durable_bytes: u64,
    unix_time_us: fn() -> f64,
}

impl RecordFile {
    pub fn begin(writer: Box<dyn RecordWriter>, unix_time_us: fn() -> f64) -> Result<Self, Failed> {
        let mut file = Self {
            writer: BufWriter::with_capacity(BUFFER_BYTES, writer),
            segment: None,
            pending_gaps: Vec::new(),
            accepted_bytes: 0,
            durable_bytes: 0,
            unix_time_us,
        };
        file.put(&format::header())?;
        file.flush()?;
        Ok(file)
    }

    pub fn durable_bytes(&self) -> u64 {
        self.durable_bytes
    }

    pub fn accept(&mut self, item: &RollItem, context: &RollContext) -> Result<Flow, Failed> {
        match item {
            RollItem::Opened => {}
            RollItem::Samples { first_index, frame } => self.samples(*first_index, frame, context)?,
            RollItem::Gap { after, missed } => self.gap(*after, *missed, context.session)?,
            RollItem::Closed { reason } => {
                if let Some(end) = EndReason::of_close(*reason) {
                    self.finish(end)?;
                    return Ok(Flow::Ended);
                }
            }
        }
        Ok(Flow::Continue)
    }

    pub fn finish(&mut self, reason: EndReason) -> Result<(), Failed> {
        self.put(&format::end(reason))?;
        self.flush()
    }

    /// Pushes the buffer to the file and checks that the file is still the one at its path.
    pub fn flush(&mut self) -> Result<(), Failed> {
        self.writer.flush().map_err(|error| Failed::of(&error))?;
        self.writer.get_mut().sync().map_err(|error| Failed::of(&error))?;
        if !self.writer.get_ref().still_linked() {
            return Err(Failed::FILE_REMOVED);
        }
        self.durable_bytes = self.accepted_bytes;
        Ok(())
    }

    fn samples(&mut self, first_index: u64, frame: &Frame, context: &RollContext) -> Result<(), Failed> {
        let Some(record) = format::samples(first_index, frame) else {
            return Ok(());
        };
        let setup = SegmentSetup::of(context, frame);
        let expected = self
            .segment
            .as_ref()
            .filter(|segment| segment.setup == setup)
            .map(|segment| segment.expected_index);
        match expected {
            Some(expected) => self.close_index_jump(first_index, expected)?,
            None => self.open_segment(first_index, setup)?,
        }
        self.put(&record)?;
        if let Some(segment) = self.segment.as_mut() {
            segment.expected_index = first_index.saturating_add(format::sample_count(frame));
        }
        Ok(())
    }

    /// Loss between modules is also a gap: the missing indices are known exactly.
    fn close_index_jump(&mut self, first_index: u64, expected: u64) -> Result<(), Failed> {
        if first_index > expected {
            self.put(&format::gap(expected.saturating_sub(1), first_index - expected, false))?;
        }
        Ok(())
    }

    fn open_segment(&mut self, first_index: u64, setup: SegmentSetup) -> Result<(), Failed> {
        let session = setup.session;
        self.put(&format::segment((self.unix_time_us)(), first_index, &setup))?;
        self.segment = Some(Segment {
            setup,
            expected_index: first_index,
        });
        for gap in std::mem::take(&mut self.pending_gaps) {
            if gap.session == session {
                self.put(&format::gap(gap.after, gap.missed, true))?;
            }
        }
        Ok(())
    }

    fn gap(&mut self, after: u64, missed: u64, session: u32) -> Result<(), Failed> {
        match self.segment.as_mut() {
            Some(segment) if segment.setup.session == session => {
                segment.expected_index = after.saturating_add(1).saturating_add(missed);
                self.put(&format::gap(after, missed, true))
            }
            _ => {
                self.pending_gaps.retain(|gap| gap.session == session);
                self.pending_gaps.push(PendingGap { session, after, missed });
                Ok(())
            }
        }
    }

    fn put(&mut self, bytes: &[u8]) -> Result<(), Failed> {
        self.writer.write_all(bytes).map_err(|error| Failed::of(&error))?;
        self.accepted_bytes += bytes.len() as u64;
        Ok(())
    }
}
