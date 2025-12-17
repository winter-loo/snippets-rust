#[cfg(any(
    all(feature = "raw_integer_pointer_cast", feature = "strict_provenance_api"),
    all(feature = "raw_integer_pointer_cast", feature = "exposed_provenance_api"),
    all(feature = "strict_provenance_api", feature = "exposed_provenance_api"),
))]
compile_error!("These pointer provenance features are mutually exclusive.");


#[cfg(feature = "raw_integer_pointer_cast")]
fn main() {
    let x = 5;
    let p = &x as *const i32;
    let q = (p as usize + 4) as *const i32;
    unsafe {
        println!("{}", *q);
    }
}

#[cfg(feature = "strict_provenance_api_pointer_misuse_detected")]
fn main() {
    let x = 5;
    let p = &x as *const i32;
    let m = &x as *const i32;
    let q = m.with_addr(p.addr() + 4) as *const i32;
    unsafe {
        println!("{}", *q);
    }
}

#[cfg(feature = "strict_provenance_api")]
fn main() {
    let arr = [10, 20, 30, 40];
    let p = arr.as_ptr();

    let addr = p.addr();
    let q = p.with_addr(addr + 4); // move to next i32

    unsafe {
        println!("{}", *q);
    }
}

#[cfg(feature = "exposed_provenance_api")]
fn main() {
    let arr = [10, 20, 30, 40];
    let p = arr.as_ptr();

    let addr = p.expose_provenance();
    let q: *const i32 = std::ptr::with_exposed_provenance(addr + 4); // move to next i32

    unsafe {
        println!("{}", *q);
    }
}

#[cfg(not(any(
    feature = "raw_integer_pointer_cast",
    feature = "strict_provenance_api",
    feature = "strict_provenance_api_pointer_misuse_detected",
    feature = "exposed_provenance_api"
)))]
fn main() {
    println!("build with different features to experiment pointer proveMance");
}
