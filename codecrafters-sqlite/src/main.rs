use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 112];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header[16], header[17]]);
            let num_tables = u16::from_be_bytes([header[103], header[104]]);

            println!("database page size: {}", page_size);
            println!("number of tables: {num_tables}");
        }
        // use https://sqlite-internal.pages.dev/ and https://www.sqlite.org/fileformat.html
        // to inspect database file format
        ".tables" => {
            let mut file = File::open(&args[1])?;
            // 100 bytes for database file header
            let mut header = [0; 100];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header[16], header[17]]);

            // 8 bytes for btree leaf page
            let mut page_header = [0; 8];
            file.read_exact(&mut page_header)?;
            let num_rows = u16::from_be_bytes([page_header[3], page_header[4]]);

            // println!("database page size: {}", page_size);
            // println!("number of tables: {num_rows}");

            let mut cell_pointers = vec![0; (num_rows * 2) as usize];
            file.read_exact(&mut cell_pointers)?;
            // println!("cell pointers: {:02x?}", cell_pointers);

            // The cell pointer array consists of K 2-byte integer offsets to the cell contents.
            for i in 0..num_rows {
                let cell_offset = u16::from_be_bytes([
                    cell_pointers[(i * 2) as usize],
                    cell_pointers[(i * 2 + 1) as usize],
                ]);
                // println!("cell offset: {cell_offset}");
                file.seek(std::io::SeekFrom::Start(cell_offset as u64))?;

                // read table B-Tree Leaf Cell Header
                let cell_payload_size = read_varint(&mut file)?;
                #[allow(unused_variables)]
                let rowid = read_varint(&mut file)?;
                // println!("rowid {rowid}");

                // read cell payload
                let mut cell_payload = vec![0; cell_payload_size as usize];
                file.read_exact(&mut cell_payload)?;

                // A record contains a header and a body, in that order.
                // The header begins with a single varint which determines the
                // total number of bytes in the header.
                //
                // The varint value is the size of the header in bytes including the size varint itself.
                let (varint_size, record_header_size) = read_varint_from_buf(&cell_payload);
                let mut varint_offset = 0;

                // Following the size varint are one or more additional varints, one per column
                varint_offset += varint_size;
                // first column info, sqlite_schema.type
                let (varint_size, type_info) = read_varint_from_buf(&cell_payload[varint_offset..]);
                // type text
                assert!(type_info >= 13 && type_info % 2 != 0);
                let type_len = (type_info - 13) / 2;

                varint_offset += varint_size;

                // second column info, sqlite_schema.name
                let (_varint_size, type_info) =
                    read_varint_from_buf(&cell_payload[varint_offset..]);

                // type text
                assert!(type_info >= 13 && type_info % 2 != 0);
                let name_len = (type_info - 13) / 2;

                let name = String::from_utf8(
                    cell_payload
                        [record_header_size + type_len..record_header_size + type_len + name_len]
                        .into(),
                )?;
                print!("{name}");
                if i + 1 != num_rows {
                    print!(" ");
                }
            }
            println!();
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}

fn read_varint(file: &mut File) -> Result<u64> {
    let mut buf = [0; 1];
    let mut sum = 0u64;
    loop {
        file.read_exact(&mut buf)?;
        let n = u8::from_be_bytes(buf);
        if n & 0x80 == 0 {
            sum += n as u64;
            break;
        }
        // read next u8
        sum += (n & 0x7f) as u64;
    }
    Ok(sum)
}

fn read_varint_from_buf(buf: &[u8]) -> (usize, usize) {
    let mut sum = 0usize;
    let mut i = 0;
    while i < buf.len() {
        let n = u8::from_be_bytes([buf[i]]);
        if n & 0x80 == 0 {
            sum += n as usize;
            i += 1;
            break;
        }
        // read next u8
        sum += (n & 0x7f) as usize;
        i += 1;
    }
    assert!(i < buf.len());
    (i, sum)
}
