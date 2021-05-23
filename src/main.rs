use hyades_core::SCTPEndpoint;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(short = "m", long = "mode")]
    pub mode: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if std::env::var("HYADES_LOG").is_err() {
        std::env::set_var("HYADES_LOG", "info");
    }
    env_logger::init();

    let options = Options::from_args();
    if &options.mode == "l" {
        info!("starting Z ...");
        let _ = SCTPEndpoint::associate_recv("127.0.0.1:6001").await;
    } else {
        info!("starting A ...");
        let _ = SCTPEndpoint::associate_send("127.0.0.1:6000", "127.0.0.1:6001", 1).await;
    }
}
