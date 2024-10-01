use std::{path::Display, time::Duration};

use chrono::Utc;
use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::*,
    service::{ipc, port_factory::publish_subscribe},
};

#[derive(Debug, Clone)]
pub struct Msg {
    pub ts: chrono::DateTime<Utc>,
    pub seq: usize,
    pub payload: MessagePayload,
}
impl Msg {
    fn new(seq: usize, payload: MessagePayload) -> Self {
        Self {
            ts: Utc::now(),
            seq,
            payload,
        }
    }
}
impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] ", self.ts.to_rfc3339())?;
        write!(f, "[Seq={}] ", self.seq)?;
        write!(f, "[{:?}]", self.payload)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MessagePayload {
    Start,
    Tick,
    End,
}

pub struct p {
    seq: usize,
    publisher: Publisher<ipc::Service, Msg, ()>,
}
impl p {
    fn new(service: publish_subscribe::PortFactory<ipc::Service, Msg, ()>) -> anyhow::Result<Self> {
        let publisher = service.publisher_builder().create()?;

        Ok(Self { publisher, seq: 0 })
    }

    pub fn send(&mut self, payload: MessagePayload) -> anyhow::Result<()> {
        let message = Msg::new(self.seq, payload);
        log::trace!("[TX] {}", message);

        let sample = self.publisher.loan_uninit()?;
        let sample = sample.write_payload(message);
        sample.send()?;
        self.seq += 1;
        Ok(())
    }
}
pub struct s {
    pub id: String,
    seq: usize,
    subscriber: Subscriber<ipc::Service, Msg, ()>,
}
impl s {
    fn new(
        id: String,
        service: publish_subscribe::PortFactory<ipc::Service, Msg, ()>,
    ) -> anyhow::Result<Self> {
        let subscriber = service.subscriber_builder().create()?;

        Ok(Self {
            id,
            subscriber,
            seq: 0,
        })
    }

    pub fn recv(&mut self) -> anyhow::Result<Vec<Msg>> {
        let mut msgs = Vec::new();
        while let Some(sample) = self.subscriber.receive()? {
            self.seq += 1;
            log::trace!("[RX] {}", *sample);
            msgs.push(sample.clone()); // TODO: Is this reqd?
        }

        Ok(msgs)
    }
}

fn service(
    name: String,
    node: &Node<ipc::Service>,
) -> anyhow::Result<publish_subscribe::PortFactory<ipc::Service, Msg, ()>> {
    Ok(node
        .service_builder(&ServiceName::new(&name)?)
        .publish_subscribe::<Msg>()
        .enable_safe_overflow(true)
        // the maximum history size a subscriber can request
        .history_size(30)
        // the maximum buffer size of a subscriber
        .subscriber_max_buffer_size(30)
        // the maximum amount of subscribers of this service
        .max_subscribers(5)
        // the maximum amount of publishers of this service
        .max_publishers(5)
        .open_or_create()?)
}

pub fn publisher(name: String, node: &Node<ipc::Service>) -> anyhow::Result<p> {
    p::new(service(name, node)?)
}

pub fn subscriber(name: String, node: &Node<ipc::Service>) -> anyhow::Result<s> {
    s::new(name.clone(), service(name, node)?)
}

pub fn setup_logging() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Utc::now().to_rfc3339(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
