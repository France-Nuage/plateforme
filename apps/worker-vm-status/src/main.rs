use tokio::time;

#[tokio::main]
async fn main() {
    env_logger::init();

    let tick = std::env::var("WORKER_HEARTBEAT_FREQUENCY")
        .unwrap_or(String::from("5"))
        .parse::<u64>()
        .expect("WORKER_HEARTBEAT_FREQUENCY could not be parsed to a number");

    let mut interval = time::interval(time::Duration::from_secs(tick));
    let client = reqwest::Client::new();

    loop {
        let result = worker::run(&client).await;
        if result.is_err() {
            let err = result.unwrap_err();
            log::error!("Error while running the worker: {:?}", err);
        }
        interval.tick().await;
        log::info!("tick!");
    }
}
