pub mod demo {
    pub mod stats {
        include!(concat!(env!("OUT_DIR"), "/demo.stats.rs"));
    }
}

use demo::stats;

fn main() {
    let revlog = stats::RevlogEntry::default();
    println!("{revlog:#?}");
}
