use mdka::from_html;

fn main() {
    let input = r#"    
    <code lang="rust"><span class="hljs-keyword">fn</span> <span class="hljs-title function_">main</span>() {
        <span class="hljs-built_in">println!</span>(<span class="hljs-string">"Hello, World!"</span>);
    }
    </code>"#;

    let input = r#"
<p>Certainly! Here's a simple "Hello, World!" program in Rust:</p>
<p>In Rust, <code>fn main()</code> is the entry point of the program, and <code>println!("Hello, World!");</code> prints "Hello, World!" to the console. To run this program, save it to a file (e.g., <code>hello.rs</code>) and compile it using the Rust compiler (<code>rustc</code>) with the command:</p>
    "#;
    let ret = from_html(input);
    println!("{}", ret);
}
