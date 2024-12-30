use bincode::Options;

fn main() {
    let i = 1u32;
    let j = 2u64;

    let mut buf: Vec<u8> = Vec::new();

    let encoder = bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding();

    let _ = encoder.serialize_into(&mut buf, &i);
    println!("buf len: {}", buf.len()); // 4

    let _ = encoder.serialize_into(&mut buf, &j);
    println!("buf len: {}", buf.len()); // 4 + 8
}
