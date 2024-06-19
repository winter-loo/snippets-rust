use std::env;
use std::fmt::Display;
#[cfg(env = "OUTPUT_ORIGIN")]
use std::io::Write;
use std::io::{self, Read, Seek, SeekFrom};

mod comment_of_c;
use comment_of_c::*;

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
        self.buffer.copy_within(self.offset..self.end, 0);
        self.file_offset += self.offset;
        self.offset = 0;
        self.end = unconsumed;

        let len = self.code_file.read(&mut self.buffer[unconsumed..]).unwrap();
        self.end += len;

        #[cfg(env = "OUTPUT_ORIGIN")]
        {
            let mut file = std::fs::OpenOptions::new()
                .append(true) // Open in append mode
                .create(true) // Create the file if it doesn't exist
                .open("out.c")
                .expect("out.c");
            file.write_all(&self.buffer).expect("write all");
        }
    }

    fn find_first_comment(&mut self) -> Option<CommentOfC> {
        let pos = find_first_c_comment(&self.buffer[self.offset..self.end]);
        if let Some((start, end)) = pos {
            let mut com = CommentOfC::new();
            com.offset = self.file_offset + self.offset + start;
            com.end = self.file_offset + self.offset + end;
            com.content.push_str(&String::from_utf8_lossy(
                &self.buffer[(self.offset + start)..(self.offset + end)],
            ));
            self.offset += end;
            Some(com)
        } else {
            None
        }
    }
}

impl Iterator for CommentExtractor {
    type Item = CommentOfC;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(com) = self.find_first_comment() {
            Some(com)
        } else {
            self.buffer_read();
            self.find_first_comment()
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
