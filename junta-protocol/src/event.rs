use junta::prelude::*;
use serde_cbor::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum EventType {
    Pub(String, Value),
    Sub(String),
    Req(String, Value),
    Res(String, Value),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Event {
    pub id: i32,
    pub event_type: EventType,
    //content: Value,
}

impl Event {
    pub fn new(id: i32, event_type: EventType) -> Event {
        Event { id, event_type }
    }
    pub fn try_from(msg: &MessageContent) -> JuntaResult<Event> {
        match msg {
            MessageContent::Binary(b) => {
                serde_cbor::from_slice(b).map_err(|e| JuntaErrorKind::Error(e.to_string()).into())
            }
            MessageContent::Text(b) => {
                serde_json::from_str(b).map_err(|e| JuntaErrorKind::Error(e.to_string()).into())
            }
        }
    }

    pub fn to_binary(&self) -> JuntaResult<Vec<u8>> {
        serde_cbor::to_vec(&self).map_err(|e| JuntaErrorKind::Error(e.to_string()).into())
    }

    pub fn to_text(&self) -> JuntaResult<String> {
        serde_json::to_string(&self).map_err(|e| JuntaErrorKind::Error(e.to_string()).into())
    }
}
