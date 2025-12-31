use std::io::{self, Write};
// [The Little Book of Rust Macros](https://lukaswirth.dev/tlborm/introduction.html)
macro_rules! four {
    ($b:block) => {
        println!("captured: block");
    };

    ($e:expr) => {
        println!("captured: expr");
    };

    ($i:ident) => {
        println!("captured: identifier");
    };

    () => {
        println!("captured: nothing")
    };

    ($t:tt) => {
        println!("captured: token tree");
    };

    ($a:expr, $b:expr) => {
        println!("r={}", $a + $b)
    };

    ($a:expr, $b:expr, $($r:expr)*) => { 
        {
            let mut o = $a + $b;
            $(o += $r;)*
            println!("output={o}");
        }
    };
}

// see https://lukaswirth.dev/tlborm/decl-macros/minutiae/hygiene.html
// https://en.wikipedia.org/wiki/Hygienic_macro
macro_rules! scope_var {
    ($a:ident) => {
        let mut $a = $a;
        $a = 3;
    };
    () => {
        let a = 3;
    };
}

fn main() {
    four!();
    four!(fn);
    four!(3 + 2);
    four!({3});
    four!(2, 3);
    four!(2   ,    3);
    four!(2, 3, 4 5);

    // see https://lukaswirth.dev/tlborm/decl-macros/minutiae/hygiene.html
    let a = 2;
    scope_var!();
    println!("a={a}");
    scope_var!(a);
    println!("a={a}");
}
