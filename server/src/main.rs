use ramsqlite_server::{config::Config, registry::Registry, server};
use std::{net::TcpListener, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::local(std::env::current_dir()?.join("data"));
    config.validate()?;
    std::fs::create_dir_all(&config.data_root)?;
    let listener = TcpListener::bind(config.listen)?;
    server::serve(
        listener,
        Arc::new(Registry::with_data_root(
            config.limits.clone(),
            config.data_root.clone(),
        )),
        config.limits.max_frame_bytes,
    )?;
    Ok(())
}
