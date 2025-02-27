use std::fs::File;
use std::io::{self, Write};

// PDF structure constants
const A4_WIDTH: f64 = 595.0; // In points (1/72 inch)
const A4_HEIGHT: f64 = 842.0; // In points (1/72 inch)

fn encoding_chinese_characters(text: &str) -> String {
    // PDF text objects with Chinese text require UTF-16BE encoding with BOM
    // 1. Convert to UTF-16BE bytes
    let mut utf16be_bytes = Vec::new();

    // Add Byte Order Mark (BOM) - FEFF in big-endian
    utf16be_bytes.push(0xFE);
    utf16be_bytes.push(0xFF);

    // Encode each character as UTF-16BE
    for c in text.chars() {
        let code_point = c as u32;

        // Handle Basic Multilingual Plane (BMP) characters (most common)
        if code_point <= 0xFFFF {
            utf16be_bytes.push(((code_point >> 8) & 0xFF) as u8);
            utf16be_bytes.push((code_point & 0xFF) as u8);
        } else {
            // Handle supplementary planes (rare in common Chinese)
            let adjusted = code_point - 0x10000;
            let high_surrogate = ((adjusted >> 10) & 0x3FF) + 0xD800;
            let low_surrogate = (adjusted & 0x3FF) + 0xDC00;

            utf16be_bytes.push(((high_surrogate >> 8) & 0xFF) as u8);
            utf16be_bytes.push((high_surrogate & 0xFF) as u8);
            utf16be_bytes.push(((low_surrogate >> 8) & 0xFF) as u8);
            utf16be_bytes.push((low_surrogate & 0xFF) as u8);
        }
    }

    // 2. Convert bytes to hex string
    let mut hex_string = String::from("<");
    for byte in utf16be_bytes {
        hex_string.push_str(&format!("{:02X}", byte));
    }
    hex_string.push('>');

    println!("Original text: {}", text);
    println!("PDF hex string: {}", hex_string);

    // Verify the string starts with the BOM (FEFF)
    println!("Starts with BOM: {}", hex_string.starts_with("<FEFF"));

    // Print the hex bytes for each character for verification
    println!("\nEncoding breakdown:");
    println!("BOM: FEFF");
    for (_, c) in text.chars().enumerate() {
        let code_point = c as u32;
        println!("'{}': {:04X}", c, code_point);
    }

    hex_string
}

// Specification: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf
//
// A PDF file consists of four main parts:
//
// Header
// Body (containing objects)
// Cross-reference table (xref)
// Trailer
fn main() -> io::Result<()> {
    // Create a new PDF file
    let mut file = File::create("resume.pdf")?;

    // PDF header
    file.write_all(b"%PDF-1.7\n")?;

    // Add a binary comment (recommended by PDF spec)
    file.write_all(b"%\xE2\xE3\xCF\xD3\n")?;

    // The body contains numbered objects that define the document structure.
    // Each object follows this format:
    //
    // ```
    // [object number] [generation number] obj
    // << ... object definition ... >>
    // endobj
    // ```

    // PDF objects
    let mut objects = Vec::new();

    // Object 1: Catalog
    // This is the root object that points to the page tree
    // `/Pages 2 0 R` references object 2 as the page tree
    objects.push(format!(
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
    ));

    // Object 2: Pages
    // This defines the page tree structure
    // `/Kids [3 0 R]` indicates there's one page (object 3)
    // `/Count 1` confirms there's just one page
    objects.push(format!(
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    ));

    // Object 3: Page
    // Defines a single page
    // `/Parent 2 0 R` references the page tree
    // `/Resources` section references two fonts:
    //   * `/F1 4 0 R` (Helvetica for English)
    //   * `/F2 5 0 R` (SimSun for Chinese)
    // `/MediaBox [0 0 595 842]` sets A4 page dimensions (in points)
    // `/Contents 6 0 R` references object 6 for the page content
    objects.push(format!("3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /MediaBox [0 0 {} {}] /Contents 6 0 R >>\nendobj\n", A4_WIDTH, A4_HEIGHT));

    // Object 4: Font (Arial for English text)
    // Defines a standard Type1 font (Helvetica)
    // `/WinAnsiEncoding` covers standard Western characters
    objects.push(format!("4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n"));

    // Object 5: Font (SimSun for Chinese text)
    // More complex font definition for Chinese characters
    // `/Subtype /Type0` indicates a composite font
    // `/Encoding /UniGB-UTF16-H` specifies UTF-16 encoding for Chinese characters
    // `/DescendantFonts` contains CIDFont information for the character mapping
    objects.push(format!("5 0 obj\n<< /Type /Font /Subtype /Type0 /BaseFont /SimSun /Encoding /UniGB-UTF16-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /SimSun /CIDSystemInfo << /Registry (Adobe) /Ordering (GB1) /Supplement 2 >> >>] >>\nendobj\n"));

    // Object 6: Content Stream
    // Contains the actual page content
    // `BT` and `ET` mark the beginning and end of text blocks
    // `/F2 24 Tf` selects font F2 (SimSun) at 24 points
    // `100 700 Td` positions the text 100 points from left, 700 points from bottom
    // `<...> Tj` draws the Chinese text (hex-encoded UTF-16BE)
    // `/F1 12 Tf` switches to font F1 (Helvetica) at 12 points
    // `0 -30 Td` moves 30 points down from current position
    // ``(by winter.loo)` Tj draws the English text
    //
    // Tf - Set font and size
    // Td - Move text position
    // Tj - Show text
    //
    // PDF uses a coordinate system where:
    // The origin (0,0) is at the bottom-left corner of the page
    // Positive x-values go right
    // Positive y-values go up
    let text = "陆冬冬的简历";
    let encoded = encoding_chinese_characters(text);

    // TODO: they not aligned with each other.
    //       Find a way to align them.
    let content = format!(
        "BT\n/F2 24 Tf\n100 700 Td\n{} Tj\n/F1 12 Tf\n0 -30 Td\n(by winter.loo) Tj\nET",
        encoded
    );

    objects.push(format!(
        "6 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        content.len(),
        content
    ));

    // Write all objects
    let mut offsets = Vec::new();

    for object in &objects {
        offsets.push(file.metadata()?.len());
        file.write_all(object.as_bytes())?;
    }

    // Write xref table
    let xref_offset = file.metadata()?.len();

    // The xref table enables random access to objects
    // `0 7` indicates 7 entries, starting from object 0
    // Each line provides:
    //   Byte offset of the object from start of file (10 digits)
    //   Generation number (5 digits)
    //   Status flag ('f' = free, 'n' = in use)
    // The first entry is special (always zeros and 'f')
    file.write_all(b"xref\n")?;
    file.write_all(format!("0 {}\n", objects.len() + 1).as_bytes())?;
    file.write_all(b"0000000000 65535 f \n")?;

    for offset in offsets {
        file.write_all(format!("{:010} 00000 n \n", offset).as_bytes())?;
    }

    // Write trailer
    // `/Size 7` indicates there are 7 objects (including object 0)
    // `/Root 1 0 R` points to the catalog (object 1)
    // `startxref` is followed by the byte offset to the xref table
    // `%%EOF` marks the end of the file
    file.write_all(b"trailer\n")?;
    file.write_all(format!("<< /Size {} /Root 1 0 R >>\n", objects.len() + 1).as_bytes())?;
    file.write_all(b"startxref\n")?;
    file.write_all(format!("{}\n", xref_offset).as_bytes())?;
    file.write_all(b"%%EOF")?;

    println!("PDF created successfully: resume.pdf");
    Ok(())
}
