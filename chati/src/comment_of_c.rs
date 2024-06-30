/// The returned tuple (start, end) represents the byte range of the comment.
/// The comment starts at position `start` and ends at position `end - 1`.
pub fn find_first_c_comment(buffer: &[u8]) -> Option<(usize, usize)> {
    let mut v = find_c_comments_internal(buffer, true);
    if v.len() == 0 {
        None
    } else {
        Some(v.remove(0))
    }
}

pub fn find_c_comments(buffer: &[u8]) -> Vec<(usize, usize)> {
    find_c_comments_internal(buffer, false)
}

fn find_c_comments_internal(buffer: &[u8], stop_on_first_found: bool) -> Vec<(usize, usize)> {
    enum State {
        BeginWithWhitespace,
        InsideCode,
        InsideSingleQuote,
        InsideEscape,
        InsideInlineComment,
        InsideSingleLineComment,
        InsideMultiLineComment,
    }

    let mut comments = Vec::new();
    let mut state = State::BeginWithWhitespace;
    let mut start = 0;
    let mut n_whitespaces = 0;

    let mut i = 0;
    while i < buffer.len() {
        match state {
            State::BeginWithWhitespace => {
                if buffer[i] == b'\'' {
                    state = State::InsideSingleQuote;
                } else if i < buffer.len() - 1 && buffer[i] == b'/' && buffer[i + 1] == b'*' {
                    state = State::InsideSingleLineComment;
                    start = i - n_whitespaces;
                    i += 1; // Move past the start of the comment
                } else if !buffer[i].is_ascii_whitespace() {
                    state = State::InsideCode;
                }

                if buffer[i] == b' ' || buffer[i] == b'\t' {
                    n_whitespaces += 1;
                } else {
                    n_whitespaces = 0;
                }
            }
            State::InsideCode => {
                if buffer[i] == b'\'' {
                    state = State::InsideSingleQuote;
                } else if buffer[i] == b'\n' {
                    state = State::BeginWithWhitespace;
                    n_whitespaces = 0;
                } else if i < buffer.len() - 1 && buffer[i] == b'/' && buffer[i + 1] == b'*' {
                    state = State::InsideInlineComment;
                    i += 1; // Move past the start of the comment
                }
            }
            State::InsideSingleQuote => {
                if buffer[i] == b'\\' {
                    state = State::InsideEscape;
                } else if buffer[i] == b'\'' {
                    state = State::InsideCode;
                }
            }
            State::InsideEscape => {
                // Always return to InsideSingleQuote after an escape sequence
                state = State::InsideSingleQuote;
            }
            State::InsideInlineComment => {
                if i < buffer.len() - 1 && buffer[i] == b'*' && buffer[i + 1] == b'/' {
                    state = State::InsideCode;
                    i += 1;
                }
            }
            State::InsideSingleLineComment => {
                if buffer[i] == b'\n' {
                    state = State::InsideMultiLineComment;
                } else if i < buffer.len() - 1 && buffer[i] == b'*' && buffer[i + 1] == b'/' {
                    state = State::InsideCode;
                    i += 1;
                }
            }
            State::InsideMultiLineComment => {
                if i < buffer.len() - 1 && buffer[i] == b'*' && buffer[i + 1] == b'/' {
                    state = State::InsideCode;
                    comments.push((start, i + 2));
                    i += 1;
                    if stop_on_first_found {
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    comments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_find_c_comments() {
        let a = "a = 3; /* inline comment */";
        assert_eq!(find_c_comments(a.as_bytes()).len(), 0);

        let a = "a = 3; /* inline comment */";
        assert_eq!(find_first_c_comment(a.as_bytes()), None);

        let a = "a = 3; /* inlien comment\nmultiline */";
        assert_eq!(find_c_comments(a.as_bytes()).len(), 0);

        let a = "/* single line comment */\na = 3;";
        assert_eq!(find_c_comments(a.as_bytes()).len(), 0);

        let a = "/* single line comment */a = 3;";
        assert_eq!(find_c_comments(a.as_bytes()).len(), 0);

        let a = "/* multi-line comment\n hello world */\na = 3;";
        let v = find_c_comments(a.as_bytes());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, 0);
        assert_eq!(v[0].1, 37);

        let a = "\t  /* multi-line comment\n hello world */\na = 3;";
        let v = find_c_comments(a.as_bytes());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, 0);
        assert_eq!(v[0].1, 40);

        let a = "\n\t  /* multi-line comment\n hello world */\na = 3;\n    /* second multi-line comment\nhello there */";
        let v = find_c_comments(a.as_bytes());
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], (1, 41));
        assert_eq!(v[1], (49, 96));

        let a = "\n\t  /* multi-line comment\n hello world */\na = 3;\n    /* second multi-line comment\nhello there */";
        let v = find_first_c_comment(a.as_bytes());
        assert_eq!(v, Some((1, 41)));

        // // Example C code buffer
        // let buffer: &[u8] = b"a = 3; /* this is an inline comment */\n/* this is a single line comment */\na = 3;\na = 3; /* this is a multiline comment\n         * but it is not prefixed with whitespaces */\n/* this is a multi-line comment without whitespaces prefix.\n   Comment continues... */\nif (a == 3) {\n    /* this is a multi-line comment and it begins with whitespaces.\n     * Comment continues...\n     */\n}";
        //
        // // Find comments
        // let comments = find_c_comments(buffer);
        //
        // // Print comments positions
        // println!("original:\n{}", String::from_utf8_lossy(buffer));
        // for (start, end) in comments {
        //     println!("Comment found from {} to {}:\n{}", start, end, String::from_utf8_lossy(&buffer[start..end]));
        // }
    }
}
