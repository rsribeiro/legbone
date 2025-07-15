use anyhow::Result;
use tokio::{
    net::TcpListener,
    task,
    sync::RwLock
};
use clap::Parser;
use legbone::{
    config,
    network::connection::Connection,
    world::{World, WorldOptions},
    Opts,
};
use std::{
    net::SocketAddr,
    sync::Arc,
    path::Path
};
use tokio_stream::{
    StreamExt,
    wrappers::TcpListenerStream
};
use log::LevelFilter;
use log4rs::{
    append::console::{
        ConsoleAppender,
        Target
    },
    config::{
        Appender,
        Config,
        Root
    },
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    config::init(Path::new("server.toml"))?;
    let config = config::CONFIG.get().unwrap();

    let log_level = match opts.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    // Build a stdout logger.
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l}):<5.5}] {m}{n}")))
        .target(Target::Stdout)
        .build();

    // Build file logger
    let logfile = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{M} [{h({l}):<5.5}] {m}{n}")))
        .append(false)
        .build("log.log")?;

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stdout
    let log_config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_level)))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("file")
                .appender("stdout")
                .build(LevelFilter::Trace),
        )?;

    // Use this to change log levels at runtime
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done
    let _handle = log4rs::init_config(log_config)?;

    log::info!("log level = {log_level:?}");
    log::info!("config = {config:?}");

    let socket_addr = SocketAddr::from((config.server.ip, config.server.port));

    if let Err(err) = legbone::map::init_map(&config.world.map) {
        panic!("Error initializing map: {err:?}");
    }

    let world = World::new();
    let world_options = WorldOptions {
        day_night_cycle_enabled: config.world.day_night_cycle,
    };

    let handle = task::spawn(game_loop(world, socket_addr, world_options));

    handle.await.expect("game loop task join")?;
    Ok(())
}

async fn game_loop(
    world: Arc<RwLock<World>>,
    socket_addr: SocketAddr,
    world_options: WorldOptions,
) -> Result<()> {
    let mut listener = TcpListenerStream::new(TcpListener::bind(socket_addr).await?);
    log::info!("Server listening on address {socket_addr}");

    let sender = {
        World::init_loop(&world, world_options);
        world.read().await.sender()
    };

    while let Some(stream) =  listener.next().await {
        let stream = stream?;

        let sender_clone = sender.clone();

        let _handle = task::spawn(async {
            log::info!("New connection: {}", stream.peer_addr().unwrap());
            match Connection::handle_login(stream, sender_clone).await {
                Ok(connection) => {
                    if let Some(mut connection) = connection {
                        if let Err(err) = connection.handle_connection().await {
                            if let Err(err) = connection.send_error(err).await {
                                log::error!("Error sending error to client: {err}");
                            }
                        }
                    }
                }
                Err(err) => log::error!("Error on client login: {err}"),
            }
        });
    }

    Ok(())
}
