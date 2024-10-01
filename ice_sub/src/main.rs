use clap::Parser;
use core::time::Duration;
use ice_shared::{setup_logging, subscriber, Msg};
use iceoryx2::prelude::*;
use std::{collections::HashMap, process::Stdio};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(long)]
    agents: usize,
}

const CLIENT: &str = "/Users/c/git/ipc_poc/target/release/ice_pub";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    setup_logging()?;

    let names = (0..args.agents)
        .map(|i| format!("testing{i}"))
        .collect::<Vec<_>>();

    let mut processes = JoinSet::new();
    for name in names.iter() {
        let mut cmd = tokio::process::Command::new(CLIENT);
        cmd.arg("--service").arg(name);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut proc = cmd.spawn()?;

        processes.spawn(async move {
            proc.wait().await.unwrap();
        });
    }

    let mut subscribers = names
        .iter()
        .map(|name| subscriber(name.clone(), &node))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let cancellation_token = CancellationToken::new();

    let ctrl_c_token = cancellation_token.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                ctrl_c_token.cancel();
            }
            Err(e) => {
                ctrl_c_token.cancel();
            }
        };
    });

    let mut seq_map: HashMap<String, usize> = HashMap::new();

    while !cancellation_token.is_cancelled() {
        for subscriber in subscribers.iter_mut() {
            let seq = seq_map.entry(subscriber.id.clone()).or_insert(0);

            let Ok(messages) = subscriber.recv() else {
                log::warn!("Failed to recv updates from subscriber {}", subscriber.id);
                continue;
            };
            for message in messages {
                if message.seq != *seq {
                    log::warn!("Mismatched sequence: Expected {seq} got {}", message.seq);
                }

                *seq += 1;
            }

            if *seq % 1000 == 0 {
                log::info!("Subscriber {}: Seq = {seq}", subscriber.id)
            }
        }
    }

    println!("{:#?}", seq_map);

    Ok(())
}
