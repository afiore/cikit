use anyhow::Result;

//TODO: delete this trait as it brings no value!
pub trait Notifier {
    type Event;
    type CIContext;
    fn notify(&mut self, event: Self::Event, ctx: Self::CIContext) -> Result<()>;
}
