fn main() {
    let a = 512;
    let b: Vec<u8> = format!("{a:b}").into_bytes().iter().map(|u| u - '0' as u8).collect();
    println!("{b:?}");
    let v: Vec<_> = (0..(7 + b.len()/7) / 7).map(|i| {
        let end = b.len() - i * 7;
        let start = end.saturating_sub(7);
        &b[start..end]
    }).collect();
    println!("{v:?}");


    let input = &[268_435_455];
    let output = to_bytes(input);
    let expected = vec![0xff, 0xff, 0xff, 0x7f];
    // assert_eq!(output, expected);

    let input = &[134_217_728];
    let output = to_bytes(input);
    let expected = vec![0xc0, 0x80, 0x80, 0x0];
    // assert_eq!(output, expected);
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    IncompleteNumber,
}

/// Convert a list of numbers to a stream of bytes encoded with variable length encoding.
pub fn to_bytes(values: &[u32]) -> Vec<u8> {
    for number in values {
        let s: Vec<u8> = format!("{number:b}").into_bytes().iter().map(|n| n - '0' as u8).collect();
        // (0..=(s.len() + 7) / 7).map(move |i| s[i..i+7].to_vec());
        // }).map(|mut s| {
        //     s.get_mut(0).map(|b| *b = 1);
        //         s
        //     }).collect();
        // v.last_mut().map(|u| {
        //     u.get_mut(0).map(|b| *b = 0);
        //     u
        // });
    }
    vec![]
}

/// Given a stream of bytes, extract all numbers which are encoded in there.
pub fn from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Error> {
    todo!("Convert the list of bytes {bytes:?} to a list of numbers")
}
