fn main() {
    FONTS.iter().for_each(|font| {
        let v: Vec<_> = font.1.iter().map(|v| std::string::String::from_utf8(v.to_vec()).unwrap()).collect();
        let v = v.join("\n");
        println!("{v}");
    });
}

mod tests {
    use super::*;
    #[test]
    fn input_with_lines_not_multiple_of_four_is_error() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "| |\n" +
                "   ";
        assert_eq!(Err(Error::InvalidRowCount(3)), convert(&input));
    }
    #[test]
    fn input_with_columns_not_multiple_of_three_is_error() {
        #[rustfmt::skip]
    let input = "    \n".to_string() +
                "   |\n" +
                "   |\n" +
                "    ";
        assert_eq!(Err(Error::InvalidColumnCount(4)), convert(&input));
    }
    #[test]
    fn unrecognized_characters_return_question_mark() {
        #[rustfmt::skip]
    let input = "   \n".to_string() +
                "  _\n" +
                "  |\n" +
                "   ";
        assert_eq!(Ok("?".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_0() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "| |\n" +
                "|_|\n" +
                "   ";
        assert_eq!(Ok("0".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_1() {
        #[rustfmt::skip]
    let input = "   \n".to_string() +
                "  |\n" +
                "  |\n" +
                "   ";
        assert_eq!(Ok("1".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_2() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                " _|\n" +
                "|_ \n" +
                "   ";
        assert_eq!(Ok("2".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_3() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                " _|\n" +
                " _|\n" +
                "   ";
        assert_eq!(Ok("3".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_4() {
        #[rustfmt::skip]
    let input = "   \n".to_string() +
                "|_|\n" +
                "  |\n" +
                "   ";
        assert_eq!(Ok("4".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_5() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "|_ \n" +
                " _|\n" +
                "   ";
        assert_eq!(Ok("5".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_6() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "|_ \n" +
                "|_|\n" +
                "   ";
        assert_eq!(Ok("6".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_7() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "  |\n" +
                "  |\n" +
                "   ";
        assert_eq!(Ok("7".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_8() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "|_|\n" +
                "|_|\n" +
                "   ";
        assert_eq!(Ok("8".to_string()), convert(&input));
    }
    #[test]
    fn recognizes_9() {
        #[rustfmt::skip]
    let input = " _ \n".to_string() +
                "|_|\n" +
                " _|\n" +
                "   ";
        assert_eq!(Ok("9".to_string()), convert(&input));
    }
}

// The code below is a stub. Just enough to satisfy the compiler.
// In order to pass the tests you can add-to or change any of this code.

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidRowCount(usize),
    InvalidColumnCount(usize),
}

//    _  _
//   | _| _|
//   ||_  _|
// 
//     _  _
// |_||_ |_
//   | _||_|
// 
//  _  _  _
//   ||_||_|
//   ||_| _|
// 
pub fn convert(input: &str) -> Result<String, Error> {
    // let matrix: Vec<_> = input
    //     .lines()
    //     .map(|line| line.bytes().collect::<Vec<_>>())
    //     .collect();
    let mut matrix = vec![vec![]; 4];
    for (i, line) in input.lines().enumerate() {
        let bytes: Vec<_> = line.bytes().collect();
        let chunks: Vec<_> = bytes.chunks(3).collect();
        if chunks.last().is_some_and(|c| c.len() != 3) {
            return Err(Error::InvalidColumnCount(2));
        }

        matrix[i].push(chunks);
    }

    if matrix.len() != 4 {
        return Err(Error::InvalidRowCount(matrix.len()));
    }
    if !matrix.iter().all(|v| v.len() == 3) {
        let line = matrix.iter().skip_while(|v| v.len() == 3).next().unwrap();
        return Err(Error::InvalidColumnCount(line.len()));
    }
    Ok(FONTS
        .iter()
        .filter(|font| matrix == font.1)
        .next()
        .map(|font| font.0.to_string())
        .unwrap_or("?".to_string()))
}

type FontMatrix = [[u8; 3]; 4];

const ZERO: FontMatrix = [
    [b' ', b'_', b' '],
    [b'|', b' ', b'|'],
    [b'|', b'_', b'|'],
    [b' ', b' ', b' '],
];

const ONE: FontMatrix = [
    [b' ', b' ', b' '],
    [b' ', b' ', b'|'],
    [b' ', b' ', b'|'],
    [b' ', b' ', b' '],
];

const TWO: FontMatrix = [
    [b' ', b'_', b' '],
    [b' ', b'_', b'|'],
    [b'|', b'_', b' '],
    [b' ', b' ', b' '],
];

const THREE: FontMatrix = [
    [b' ', b'_', b' '],
    [b' ', b'_', b'|'],
    [b' ', b'_', b'|'],
    [b' ', b' ', b' '],
];

const FOUR: FontMatrix = [
    [b' ', b' ', b' '],
    [b'|', b'_', b'|'],
    [b' ', b' ', b'|'],
    [b' ', b' ', b' '],
];

const FIVE: FontMatrix = [
    [b' ', b'_', b' '],
    [b'|', b'_', b' '],
    [b' ', b'_', b'|'],
    [b' ', b' ', b' '],
];

const SIX: FontMatrix = [
    [b' ', b'_', b' '],
    [b'|', b'_', b' '],
    [b'|', b'_', b'|'],
    [b' ', b' ', b' '],
];

const SEVEN: FontMatrix = [
    [b' ', b'_', b' '],
    [b' ', b' ', b'|'],
    [b' ', b' ', b'|'],
    [b' ', b' ', b' '],
];

const EIGHT: FontMatrix = [
    [b' ', b'_', b' '],
    [b'|', b'_', b'|'],
    [b'|', b'_', b'|'],
    [b' ', b' ', b' '],
];

const NINE: FontMatrix = [
    [b' ', b'_', b' '],
    [b'|', b'_', b'|'],
    [b' ', b'_', b'|'],
    [b' ', b' ', b' '],
];

const FONTS: [(usize, FontMatrix); 10] = [
    (0, ZERO),
    (1, ONE),
    (2, TWO),
    (3, THREE),
    (4, FOUR),
    (5, FIVE),
    (6, SIX),
    (7, SEVEN),
    (8, EIGHT),
    (9, NINE),
];
