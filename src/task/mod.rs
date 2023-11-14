mod init_peer;
use init_peer::run as init_peer_run;

use std::time::Duration;

use log::info;
use tokio::time::sleep;

pub async fn task_loop(time: u64) {
    sleep(Duration::from_secs(time)).await;
    // One time task
    init_peer_run().await;

    loop {
        // Multiple Time Task
        info!("Task Looping");
        
        sleep(Duration::from_secs(20)).await;
    }
}

pub async fn init() {
    tokio::spawn(task_loop(10));
}
