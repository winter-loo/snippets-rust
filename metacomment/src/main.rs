use std::env;
use std::fmt::Display;
use std::io::{self, Read, Seek, SeekFrom, Write};

struct CommentOfC {
    offset: usize,
    end: usize,
    content: String,
    pre_whitespaces: String,
}

impl Display for CommentOfC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "comment <{}:{}> ---", self.offset, self.end)?;
        write!(f, "{}{}", self.pre_whitespaces, self.content)
    }
}

impl CommentOfC {
    fn new() -> Self {
        CommentOfC {
            offset: 0,
            end: 0,
            content: String::from(""),
            pre_whitespaces: String::from(""),
        }
    }

    fn reset(&mut self) {
        self.offset = 0;
        self.end = 0;
        self.content.clear();
        self.pre_whitespaces.clear();
    }
}

struct CommentExtractor {
    code_file: std::fs::File,
    buffer: [u8; 4096],
    offset: usize,
    end: usize,
    file_offset: usize,
}

impl CommentExtractor {
    fn new(code_file: std::fs::File) -> Self {
        let mut myself = CommentExtractor {
            code_file,
            buffer: [0; 4096],
            offset: 0,
            end: 0,
            file_offset: 0,
        };
        myself
            .code_file
            .seek(SeekFrom::Start(0))
            .expect("seek to start");

        myself
    }

    fn buffer_read(&mut self) {
        let unconsumed = self.end - self.offset;
        println!("-- unconsumed {}", unconsumed);
        self.buffer.copy_within(self.offset..self.end, 0);
        self.file_offset += self.offset;
        self.offset = 0;
        self.end = unconsumed;

        let len = self.code_file.read(&mut self.buffer[unconsumed..]).unwrap();
        println!("buffer_read len: {}", len);
        self.end += len;

        let mut file = std::fs::OpenOptions::new()
            .append(true) // Open in append mode
            .create(true) // Create the file if it doesn't exist
            .open("out.c").expect("out.c");
        file.write_all(&self.buffer).expect("write all");

    }

    fn find_comment(&mut self) -> Option<CommentOfC> {
        let mut com = CommentOfC::new();
        let mut found_comment = false;
        let mut i = self.offset;
        let mut maybe_inline_comment = false;
        let mut pre_whitespaces = String::from("");

        while i < self.end {
            if i + 2 < self.end
                && (self.buffer[i] != b'\'' && self.buffer[i] != b'"')
                && self.buffer[i + 1] == b'/'
                && self.buffer[i + 2] == b'*' || (self.file_offset == 0 && i == 0 && i + 1 < self.end && self.buffer[i] == b'/' && self.buffer[i + 1] == b'*')
            {
                if self.buffer[i] == b'/' {
                    com.offset = self.file_offset + i;
                } else {
                    com.offset = self.file_offset + i + 1;
                }
                com.content.push('/');
                com.content.push('*');

                if self.buffer[i] == b' ' || self.buffer[i] == b'\t' {
                    pre_whitespaces.push(self.buffer[i] as char);
                }

                if self.buffer[i] != b'/' {
                    i += 1
                }

                if !maybe_inline_comment {
                    com.pre_whitespaces = pre_whitespaces.clone();
                } else {
                    // we do not process inline comments
                    i += 2;
                    continue;
                }

                let mut multiline = false;

                i += 2; // Skip past the '/*'
                while i + 1 < self.end {
                    if self.buffer[i] == b'*' && self.buffer[i + 1] == b'/' {
                        com.end = self.file_offset + i + 2;
                        if multiline {
                            found_comment = true;
                            com.content.push('*');
                            com.content.push('/');

                            pre_whitespaces.clear();
                            break;
                        } else {
                            com.reset();
                            break;
                        }
                    } else {
                        if self.buffer[i] == b'\n'
                            || self.buffer[i] == b'\r'
                            || (i + 1 < self.end
                                && self.buffer[i] == b'\r'
                                && self.buffer[i] == b'\n')
                        {
                            multiline = true;
                        }
                        com.content.push(self.buffer[i] as char);
                        i += 1;
                    }
                }
            }
            if self.buffer[i] == b'\n'
                || self.buffer[i] == b'\r'
                || (i + 1 < self.end && self.buffer[i] == b'\r' && self.buffer[i] == b'\n')
            {
                maybe_inline_comment = false;
                pre_whitespaces.clear();
            } else if self.buffer[i] != b' ' && self.buffer[i] != b'\t' {
                maybe_inline_comment = true;
            } else {
                maybe_inline_comment = false;
                pre_whitespaces.push(self.buffer[i] as char);
            }
            if found_comment {
                break;
            }
            i += 1;
        }

        if found_comment {
            self.offset = com.end + 1;
            Some(com)
        } else {
            self.offset = self.end;
            None
        }
    }
}

impl Iterator for CommentExtractor {
    type Item = CommentOfC;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(com) = self.find_comment() {
            Some(com)
        } else {
            self.buffer_read();
            self.find_comment()
        }
    }
}

fn main() -> io::Result<()> {
    let mut args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <c code file>", args[0]);
        return Ok(());
    }
    let infile = args.remove(1);
    let infile = std::fs::File::open(infile)?;
    let ce = CommentExtractor::new(infile);
    for com in ce {
        println!("{}", com);
    }
    Ok(())
}
