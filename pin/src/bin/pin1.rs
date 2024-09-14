use std::pin::Pin;

#[derive(Debug)]
struct Foo {
    // In somewhat way, we establish the invariant:
    // `a` always points to itself. i.e. `a` represents the start
    // memory address of current Foo instance
    a: *const Foo,
    b: u8,
}

impl Foo {
    fn new(i: u8) -> Foo {
        Foo {
            a: std::ptr::null_mut(),
            b: i,
        }
    }

    fn get_b_from_a(&self) -> u8 {
        let mya = unsafe { &*self.a };
        mya.b
    }

    fn check_invariant(&self) {
        assert_eq!(self.a, self as *const Foo);
    }
}

fn handle_foo(f1: &mut Pin<&mut Foo>, f2: &mut Pin<&mut Foo>) {
    let f1 = f1.as_mut().get_mut();
    f1.a = f2.a;
    // std::mem::swap(f1, f2);
}

fn debug_memory(f1: &Foo, f2: &Foo) {
    println!("f1 addr: {:p}", &f1);
    println!("f1: {:#?}", f1);
    println!("f2 addr: {:p}", &f2);
    println!("f2: {:#?}", f2);
    println!("f1.a.b: {}, f1.b: {}", f1.get_b_from_a(), f1.b);
    println!("f2.a.b: {}, f2.b: {}", f2.get_b_from_a(), f2.b);
}

fn main() {
    let mut f1 = Foo::new(10); 
    f1.a = &f1 as *const Foo;

    let mut f2 = Foo::new(20);
    f2.a = &f2 as *const Foo;

    debug_memory(&f1, &f2);

    f1.check_invariant();
    f2.check_invariant();

    // change f1, f2 to Pin pointer
    let mut f1 = unsafe { Pin::new_unchecked(&mut f1) };
    let mut f2 = unsafe { Pin::new_unchecked(&mut f2) };

    // where we do the dangerous things
    handle_foo(&mut f1, &mut f2);

    debug_memory(&f1, &f2);

    // after that, we hope our invarint hold
    f1.check_invariant();
    f2.check_invariant();
}
