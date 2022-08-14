use std::{
    sync::{
        Arc,
        RwLock
    }
};
use async_std::{
    prelude::*,
    net::{
        TcpListener,
        SocketAddr
    },
    task
};
use legbone::{
    network::connection::Connection,
    world::{
        World,
        WorldOptions
    },
    Opts,
};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let log_level = match opts.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .filter(Some("legbone"), log_level)
            .filter(Some("async_std"), log::LevelFilter::Error)
            .filter(Some("polling"), log::LevelFilter::Error)
            .init();

    log::info!("log level = {:?}", log_level);

    let socket_addr = SocketAddr::from((opts.ip, opts.port));

    if let Err(err) = legbone::map::init_map(opts.map, opts.map_arg) {
        panic!("Error initializing map: {:?}", err);
    }

    let world = World::new();
    let world_options = WorldOptions {
        day_night_cycle_enabled: opts.day_night_cycle_enabled
    };

    task::block_on(game_loop(world, socket_addr, world_options))
}

async fn game_loop(world: Arc<RwLock<World>>, socket_addr: SocketAddr, world_options: WorldOptions) -> Result<()> {
    let listener = TcpListener::bind(socket_addr).await?;
    log::info!("Server listening on address {}", socket_addr);

    let sender = {
        World::init_loop(&world, world_options);
        world.read().unwrap().sender()
    };

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;

        let sender_clone = sender.clone();

        let _handle = task::spawn(async move {
            log::info!("New connection: {}", stream.peer_addr().unwrap());
            match Connection::handle_login(stream, sender_clone).await {
                Ok(connection) => {
                    if let Some(mut connection) = connection {
                        if let Err(err) = connection.handle_connection().await {
                            if let Err(err) = connection.send_error(err).await {
                                log::error!("Error sending error to client: {}", err);
                            }
                        }
                    }
                },
                Err(err) => log::error!("Error on client login: {}", err)
            }
        });
    }
    Ok(())
}
