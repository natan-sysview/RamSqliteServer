use ramsqlite_server::{config::Config, registry::Registry, server};
use std::{env, net::TcpListener, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_root = env::var_os("RAMSQLITE_DATA_ROOT")
        .map(Into::into)
        .unwrap_or(std::env::current_dir()?.join("data"));
    let mut config = Config::local(data_root);
    if let Ok(listen) = env::var("RAMSQLITE_LISTEN") {
        config.listen = listen.parse()?;
    }
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
