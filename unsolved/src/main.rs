use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,
    #[serde(flatten)]
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessagePayload {
    Init(InitRequest),
    InitOk,
    Echo(EchoRequest),
    EchoOk(EchoResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoRequest {
    pub echo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub echo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitRequest {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u32,
    pub text: String,
}

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("unknown message type")]
    UnknownMessageType,
    #[error("internal error: {0}")]
    Internal(String),
}

pub trait MessageHandler {
    fn can_handle(&self, req: &MessagePayload) -> bool;
    fn handle(&self, node: &mut Node, req: &MessagePayload) -> Result<MessagePayload, NodeError>;
}

pub struct InitHandler;

impl MessageHandler for InitHandler {
    fn can_handle(&self, req: &MessagePayload) -> bool {
        matches!(req, MessagePayload::Init(_))
    }

    fn handle(&self, node: &mut Node, req: &MessagePayload) -> Result<MessagePayload, NodeError> {
        if let MessagePayload::Init(init) = req {
            node.id = init.node_id.clone();
            node.node_ids = init.node_ids.clone();
            Ok(MessagePayload::InitOk)
        } else {
            Err(NodeError::UnknownMessageType)
        }
    }
}

pub struct EchoHandler;

impl MessageHandler for EchoHandler {
    fn can_handle(&self, req: &MessagePayload) -> bool {
        matches!(req, MessagePayload::Echo(_))
    }

    fn handle(&self, _node: &mut Node, req: &MessagePayload) -> Result<MessagePayload, NodeError> {
        if let MessagePayload::Echo(echo) = req {
            Ok(MessagePayload::EchoOk(EchoResponse {
                echo: echo.echo.clone(),
            }))
        } else {
            Err(NodeError::UnknownMessageType)
        }
    }
}

pub struct Node {
    pub id: String,
    pub node_ids: Vec<String>,
    message_handlers: Vec<Box<dyn MessageHandler>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: String::new(),
            node_ids: Vec::new(),
            message_handlers: vec![Box::new(InitHandler), Box::new(EchoHandler)],
        }
    }

    pub fn handle_message(&mut self, req: &Message) -> Message {
        let mut res = Message {
            src: self.id.clone(),
            dest: req.src.clone(),
            body: MessageBody {
                msg_id: req.body.msg_id.map(|id| id + 1),
                in_reply_to: req.body.msg_id,
                payload: MessagePayload::Error(ErrorResponse {
                    code: 1,
                    text: "Internal error".to_string(),
                }),
            },
        };

        let handler = self
            .message_handlers
            .iter()
            .find(|h| h.can_handle(&req.body.payload));

        if let Some(handler) = handler {
            match handler.handle(self, &req.body.payload) {
                Ok(payload) => res.body.payload = payload,
                Err(err) => {
                    res.body.payload = MessagePayload::Error(ErrorResponse {
                        code: 1,
                        text: err.to_string(),
                    });
                }
            }
        }

        res
    }

    pub fn send(&self, res: Message) -> Result<(), serde_json::Error> {
        serde_json::to_writer(std::io::stdout(), &res)?;
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin().lock();
    let mut node = Node::new();
    
    let reader = std::io::BufReader::new(stdin);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);

    loop {
        match Message::deserialize(&mut deserializer) {
            Ok(msg) => {
                let response = node.handle_message(&msg);
                node.send(response)?;
            }
            Err(err) if err.is_eof() => break,
            Err(err) => eprintln!("Error deserializing message: {}", err),
        }
    }

    Ok(())
}