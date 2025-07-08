use too_many_lists::first;

fn main() {
    // stack overflow when release memory using built-in drop mechanism
    // Hence, a manual drop method must be implemented for first::List.
    {
        let mut flist = first::List::new();
        for i in 0..100000 {
            flist.push(i);
        }
    }
}
