use chati::comment_extractor::CommentExtractor;
use chati::util::merge_comments;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <code_file> <comment_file> <merged_file>",
            args[0]
        );
        std::process::exit(1);
    }

    let code_filename = &args[1];
    let comm_filename = &args[2];
    let merg_filename = &args[3];

    let code_file = File::open(code_filename).expect("open {code_filename}");
    let comm_file = File::open(comm_filename).expect("open {comm_filename}");
    let mut merg_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(merg_filename)
        .expect("open {merg_filename}");

    let mut ce1 = CommentExtractor::new(code_file);
    let mut ce2 = CommentExtractor::new(comm_file);

    let mut out_offset = 0;
    loop {
        let com1 = ce1.next();
        if com1.is_none() {
            break;
        }
        let com1 = com1.unwrap();
        let com2 = ce2.next().expect("the same count comments");

        let merged = merge_comments(&com2.content, &com1.content);

        copy_rest(
            out_offset,
            com1.offset as u64,
            code_filename,
            &mut merg_file,
        );
        // com1.end is not inclusive
        out_offset = com1.end as u64;

        merg_file
            .write_all(merged.as_bytes())
            .expect("write to {merg_filename}");
    }
    copy_rest(out_offset, u64::MAX, code_filename, &mut merg_file);
}

/// read content from `infilename` at offset `start` until `end` and append the content to `outfile`
fn copy_rest(start: u64, end: u64, infilename: &str, outfile: &mut File) {
    let total_bytes = (end - start) as usize;
    if total_bytes == 0 {
        return;
    }

    let mut infile = File::open(infilename).expect("open {infilename}");
    infile
        .seek(SeekFrom::Start(start))
        .expect("seek to offset {start} in {infilename}");
    let mut buffer = vec![0; 4096];
    let mut nbytes = 0;
    loop {
        let n = infile.read(&mut buffer).expect("reading {infilename}");
        if n == 0 {
            break;
        }
        nbytes += n;
        if nbytes >= total_bytes {
            outfile
                .write_all(&buffer[0..n - (nbytes - total_bytes)])
                .expect("appending to file");
            break;
        }
        outfile.write_all(&buffer[0..n]).expect("appending to file");
    }
}
