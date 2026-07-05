use tokio::sync::mpsc;
use crate::tools::find_image::sleep_for;

pub async fn runit() {
    
    let (tx, mut rx) = mpsc::channel(1);
    
    tokio::spawn(async move {

        let invoice = ["I260014134", "I260014135", "I260014133", "I260014132"];
        for i in invoice {
            if let Err(_) = tx.send(i).await {
                println!("receiver dropped");
                return;
            }
        }
        
    });
    
    while let Some(i) = rx.recv().await {
        sleep_for(2);
        println!("got = {}", i);
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