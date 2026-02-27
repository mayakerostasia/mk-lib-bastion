mod event;
// mod receiver;
mod emitter;

// New event system modules
pub mod capture;
pub mod event_new;
pub mod filter;
pub mod sink;

pub use emitter::Emitter;
pub use event::SimianEvent;
pub use event::SimianEventType;

// Re-export new event system
pub use event_new::{Event, EventCategory, EventData, EventSeverity};
pub use capture::EventCapture;
pub use sink::{EventSink, MemoryEventSink, FileEventSink};
pub use filter::{EventFilter, AllowAllFilter, SeverityFilter};
