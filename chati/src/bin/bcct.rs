// block code comment translator

use chati::{chati::Chati, comment_extractor::CommentExtractor};
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use tokio::io::AsyncWriteExt;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <code_file>", args[0]);
        std::process::exit(1);
    }

    let code_file = std::fs::File::open(&args[1]).unwrap();
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(block_code_comment_translator(code_file));
}

async fn block_code_comment_translator(code_file: std::fs::File) {
    let mut ci = Chati::new().await;
    ci.new_converstation(false).await;

    let command = r#"Suppose you are a specialized code comment translator.
Translate code comments I will provide to you coming from database project in following conversations into Chinese.
You need obey strictly the following rules:

* The Chinese text should be placed in an independent comment of which style is the same as the original.
* You should not output the original comment.
* For some code-related names, say data structure name, function name, variable name, you should not translate it.
* For some English terms which may have different meaning in regular English context, you should also not translate it.
* The word in all upper case letters, you should not translate it.
* Keep the original whitespaces at the head of every line.
* The output should be in a c code block.
* Wrap the line at around the column position 80.
* Add one space between Chinese text and English text.

And remember the following translation rules:
* 'cursor' should be translated as '游标'
* 'shard' should be translated as '分片'
* 'helper' should be translated as '辅助'

And do not translate the word/words below within a sentence:
  * placement
  * colocation,
  * colocation id
  * colocation group

The final translation should preserve the structure and meaning of the original comment in Chinese."#;

    ensure_responded(&mut ci, &command, false).await;

    let ce = CommentExtractor::new(code_file);

    for com in ce {
        ensure_responded(&mut ci, &com.content, true).await;
    }

    println!("DONE");
    ci.end().await;
}

async fn append_to_file(words: &str) {
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("translated.txt")
        .await
        .expect("append to translated.txt");
    let _ = file.write_all(words.as_bytes()).await;
}

async fn ensure_responded(ci: &mut Chati, isaid: &str, remember_he_said: bool) {
    loop {
        chati::util::pause().await;
        println!("I SAID: {isaid}");
        tokio::io::stdout().flush().await.unwrap();

        ci.isaid(isaid).await;

        print!("HE SAID: ");
        tokio::io::stdout().flush().await.unwrap();

        let repeat = Arc::new(AtomicBool::new(false));
        ci.hesaid(|words| {
            let repeat = Arc::clone(&repeat);
            async move {
                match words {
                    Some(words) => {
                        // print!("{words}");
                        // let _ = tokio::io::stdout().flush().await;
                        if remember_he_said {
                            append_to_file(&words).await;
                        }
                    }
                    None => {
                        println!("he said nothing. I will repeat my said");
                        repeat.store(true, Ordering::Relaxed);
                    }
                }
            }
        })
        .await;

        if !repeat.load(Ordering::Relaxed) {
            // println!();
            // tokio::io::stdout().flush().await.unwrap();
            append_to_file("\n").await;
            break;
        }
    }
}
