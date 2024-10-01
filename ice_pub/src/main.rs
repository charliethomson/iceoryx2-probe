use clap::Parser;
use core::time::Duration;
use ice_shared::{publisher, setup_logging, Msg};
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, Parser)]
struct Args {
    #[arg(short, long)]
    service: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logging()?;
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let mut p = publisher(args.service.clone(), &node)?;

    p.send(ice_shared::MessagePayload::Start)?;

    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        p.send(ice_shared::MessagePayload::Tick)?;
    }

    p.send(ice_shared::MessagePayload::End)?;

    Ok(())
}
