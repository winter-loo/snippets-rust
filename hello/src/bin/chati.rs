use tokio::io::AsyncWriteExt;

use chati::chati::Chati;

#[tokio::main]
async fn main() {
    let mut ci = Chati::new().await;

    ci.new_converstation().await;

    let what_i_said = "hello world";
    println!("I SAID: {what_i_said}");
    tokio::io::stdout().flush().await.unwrap();

    ci.isaid(what_i_said).await;

    print!("HE SAID: ");
    tokio::io::stdout().flush().await.unwrap();

    ci.hesaid(|words| async move {
        print!("{words}");
        let _ = tokio::io::stdout().flush().await;
    })
    .await;

    println!("\ndone");
    tokio::io::stdout().flush().await.unwrap();

    ci.end().await;
}
