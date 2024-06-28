use std::io::Write;

use chati::chati::Chati;

#[tokio::main]
async fn main() {
    let mut ci = Chati::new().await;

    let what_i_said = "hello world";
    println!("I SAID: {what_i_said}");
    ci.isaid(what_i_said).await;

    print!("HE SAID: ");
    ci.hesaid(|words| {
        print!("{words}");
        let _ = std::io::stdout().flush();
    }).await;
    println!();
}
