use lambda::{handler_fn, Context};
use serde_json::{Value};
use simple_logger::{SimpleLogger};
use log::{self, info, LevelFilter};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(event: Value, _: Context) -> Result<Value, Error> {
    info!("Hello events!");
    Ok(event)
}
