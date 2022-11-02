mod stats;

use std::ops::{Sub};
use std::sync::Mutex;
use std::time::SystemTime;
pub use anyhow;
pub use discord_sdk as ds;
pub use tokio;
pub use tracing;
pub use nix;
use sysinfo::SystemExt;

extern crate byte_unit;

use byte_unit::Byte;
use once_cell::sync::Lazy;
use rand::Rng;
use tokio::task;
use crate::stats::Statistic;

pub const APP_ID: ds::AppId = 1037339945649590322;

pub struct Client {
    pub discord: ds::Discord,
    pub user: ds::user::User,
    pub wheel: ds::wheel::Wheel,
}

pub async fn make_client(subs: ds::Subscriptions) -> Client {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let (wheel, handler) = ds::wheel::Wheel::new(Box::new(|err| {
        tracing::error!(error = ?err, "encountered an error");
    }));

    let mut user = wheel.user();

    let discord = ds::Discord::new(ds::DiscordApp::PlainId(APP_ID), subs, Box::new(handler))
        .expect("unable to create discord client");

    tracing::info!("waiting for handshake...");
    user.0.changed().await.unwrap();

    let user = match &*user.0.borrow() {
        ds::wheel::UserState::Connected(user) => user.clone(),
        ds::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {:?}", err),
    };

    tracing::info!("connected to Discord, local user is {:#?}", user);

    Client {
        discord,
        user,
        wheel,
    }
}

static REGISTERED_STATS: Vec<u8> = Vec::new();

// static REGISTERED_STATS: Vec<dyn DiscordStatistic> = Vec::new();

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = make_client(ds::Subscriptions::ACTIVITY).await;

    let mut activity_events = client.wheel.activity();
    tokio::task::spawn(async move {
        while let Ok(ae) = activity_events.0.recv().await {
            tracing::info!(event = ?ae, "received activity event");
        }
    });

    // let discordc = client.discord.();

    let update_details = task::spawn(async move {
        loop {
            let rp = ds::activity::ActivityBuilder::new()
                .details(get_random_stat())
                .assets(
                    ds::activity::Assets::default()
                        .large("arch_linux_logo_round".to_owned(), Some("Arch linux".to_owned()))
                )
                .start_timestamp(SystemTime::now().sub(std::time::Duration::from(nix::time::clock_gettime(nix::time::ClockId::CLOCK_MONOTONIC).expect("bad"))));
            tracing::info!(
                "updated activity: {:?}",
                client.discord.update_activity(rp).await
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });


    // let mut r = String::new();
    // let _ = std::io::stdin().read_line(&mut r);
    //
    // update_details.abort();
    //
    // tracing::info!(
    //     "cleared activity: {:?}",
    //     client.discord.clear_activity().await
    // );
    //
    // client.discord.disconnect().await;
    update_details.await;
    Ok(())
}

fn get_random_stat() -> String {
    let mut rng = rand::thread_rng();
    let rand = rng.gen_range(0..4);

    match rand {
        0 => Statistic::RamUsage,
        1 => Statistic::ProcessCount,
        2 => Statistic::DiskSpace,
        3 => Statistic::KernelVersion,
        _ => panic!("Undefined")
    }.show()
}
