use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(serde::Deserialize, Debug)]
pub struct Conversation {
    pub message: Message,
    pub conversation_id: String,
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub author: Option<Author>,
    pub create_time: Option<f64>,
    pub update_time: Option<f64>,
    pub content: Content,
    pub status: Option<String>,
    pub end_turn: Option<bool>,
    pub weight: Option<f64>,
    pub metadata: Option<Metadata>,
    pub recipient: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Author {
    pub role: Option<String>,
    pub name: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Content {
    pub content_type: String,
    pub parts: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Metadata {
    pub citations: Option<Vec<serde_json::Value>>,
    pub gizmo_id: Option<String>,
    pub message_type: Option<String>,
    pub model_slug: Option<String>,
    pub default_model_slug: Option<String>,
    pub pad: Option<String>,
    pub parent_id: Option<String>,
    pub model_switcher_deny: Option<Vec<String>>,
}

/// handle service-sent events coming from https://chatgpt.com/backend-anon/conversation
pub fn assistant_sse(data: &str, mut outfn: impl FnMut(&str, bool)) {
    static mut CONTENT_OFFSET: usize = 0;
    static mut MESSAGE_ENDED: bool = false;
    lazy_static! {
        static ref MESSAGE_ID: Mutex<String> = Mutex::new(String::from(""));
    }

    if data.len() == 0 {
        return;
    }
    // println!("BEGIN--");
    // println!("{data}");
    // println!("--END");

    let conversations = data
        .split("\n\n")
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|line| line.len() > 0 && line.trim() != "data: [DONE]")
        .map(|line| {
            match serde_json::from_str::<Conversation>(line.trim_start_matches("data:").trim()) {
                Ok(x) => Some(x),
                Err(error) => {
                    println!("when parsing json text as Conversation: {line} {error:#?}");
                    None
                }
            }
        });

    for con in conversations {
        if con.is_none() {
            continue;
        }
        let con = con.unwrap();

        // each part message contains previous part message
        let cont_part = &con.message.content.parts[0];

        // println!("message id: {}", con.message.id);
        // let _ = std::io::stdout().flush();

        let mut msgid = MESSAGE_ID.lock().unwrap();
        let offset = if *msgid == con.message.id {
            let old_offset = unsafe { CONTENT_OFFSET };
            unsafe {
                CONTENT_OFFSET = cont_part.len();
            }
            old_offset
        } else {
            *msgid = con.message.id;
            unsafe {
                CONTENT_OFFSET = cont_part.len();
            }
            unsafe {
                MESSAGE_ENDED = false;
            }
            0
        };

        // openai could send multiple events of "message.status = 'finished_successfully'"
        // we need only one 'ended' notification
        let ended = if let Some(status) = con.message.status {
            status == "finished_successfully"
        } else {
            false
        };

        // println!("offset: {offset}, ended: {ended}");

        let msg_ended = unsafe { MESSAGE_ENDED };
        if !msg_ended {
            outfn(&cont_part[offset..], ended);
        }
        if ended {
            unsafe {
                MESSAGE_ENDED = true;
            }
        }
    }
}
