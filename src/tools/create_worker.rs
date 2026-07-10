use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use crate::tools::find_image::sleep_for;

#[derive(Debug)]
pub struct WorkerJob {
    pub invoice: String, // pub app: App, should be App struct for that which also tell's if the app take input.
    pub response_to: oneshot::Sender<String>
}

impl WorkerJob {
    fn new(invoice: String, response: Sender<String>) -> Self {
        Self { invoice, response_to: response }
    }
}

pub async fn runit() {
    
    let (tx, mut rx) = mpsc::channel(1); 
    
    let (os_tx, os_rx) = oneshot::channel::<String>();

    tokio::spawn(async move {
        let job = WorkerJob::new("I260014134".to_string(), os_tx);
        if let Err(_) = tx.send(job).await {
            println!("receiver dropped");
            return;
        }
    });
    
    while let Some(i) = rx.recv().await {
        sleep_for(2);
        let results = String::from("someBase64KindOfSheeeeet");
        if i.response_to.send(results).is_ok() {}
    }

    if let Ok(result) = os_rx.await {
            println!("{}", result);
    }

}

#[cfg(test)]
mod tests {
    use crate::tools::create_worker::runit;

    
    #[tokio::test]
    async fn run_for_test() {
        runit().await;
    }
}