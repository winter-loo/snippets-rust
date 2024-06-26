struct Chati {

}

impl Chati {
    fn new() -> Self {
        Chati {

        }
    }
}

#[tokio::main]
fn main() {
    let ci = Chati;

    let what_i_said = "hello world";
    println!("ME: {what_i_said}");
    let response = ci.chat(what_i_said).await;
    println!("CI: {response}");

    let what_i_said = "Thank you!";
    println!("ME: {what_i_said}");
    let response = ci.chat(what_i_said).await;
    println!("CI: {response}");

    let what_i_said = "Ok. That's good";
    println!("ME: {what_i_said}");
    let response = ci.chat(what_i_said).await;
    println!("CI: {response}");

    let what_i_said = "Fine. Bye!";
    println!("ME: {what_i_said}");
    let response = ci.chat(what_i_said).await;
    println!("CI: {response}");
}
