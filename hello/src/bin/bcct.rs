// block code comment translator

use chati::chati::Chati;
use chati::comment_extractor::CommentExtractor;
use tokio::io::AsyncWriteExt;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <code_file>", args[0]);
        std::process::exit(1);
    }

    let code_file = std::fs::File::open(&args[1]).unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(block_code_comment_translator(code_file));
}

async fn block_code_comment_translator(code_file: std::fs::File) {
    let ce = CommentExtractor::new(code_file);

    let mut ci = Chati::new().await;
    ci.new_converstation().await;

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
    println!("I SAID: {command}");
    tokio::io::stdout().flush().await.unwrap();

    ci.isaid(command).await;

    print!("HE SAID: ");
    tokio::io::stdout().flush().await.unwrap();

    ci.hesaid(|words| async move {
        print!("{words}");
        let _ = tokio::io::stdout().flush().await;
    })
    .await;

    println!();
    tokio::io::stdout().flush().await.unwrap();

    for com in ce {
        ci.isaid(&com.content).await;

        ci.hesaid(|words| async move {
            print!("{words}");
            let _ = tokio::io::stdout().flush().await;
        })
        .await;
    }

    ci.end().await;
}