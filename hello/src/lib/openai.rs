use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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

pub fn assistant_sse(data: &str, outfn: impl Fn(&str, bool)) {
    static CONTENT_OFFSET: AtomicUsize = AtomicUsize::new(0);
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
            serde_json::from_str::<Conversation>(line.trim_start_matches("data:").trim())
                .expect("Conversation")
        });

    for con in conversations {
        let cont_part = &con.message.content.parts[0];

        let mut msgid = MESSAGE_ID.lock().unwrap();
        let offset = if *msgid == con.message.id {
            CONTENT_OFFSET.fetch_add(
                cont_part.len() - CONTENT_OFFSET.load(Ordering::Relaxed),
                Ordering::Relaxed,
            )
        } else {
            CONTENT_OFFSET.store(cont_part.len(), Ordering::Relaxed);
            *msgid = con.message.id;
            0
        };

        outfn(
            &cont_part[offset..],
            con.message.status.is_some() && con.message.status.unwrap() == "finished_successfully",
        );
    }
}
