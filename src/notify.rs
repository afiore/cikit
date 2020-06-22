use anyhow::Result;

pub trait Notifier {
    type Event;
    type CIContext;
    fn notify(&mut self, event: Self::Event, ctx: Self::CIContext) -> Result<()>;
}
