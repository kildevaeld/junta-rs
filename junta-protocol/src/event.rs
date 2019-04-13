use junta::prelude::*;
use serde_cbor::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum EventType {
    Pub(String, Value),
    Sub(String),
    Unsub(String),
    Req(String, Value),
    Res(String, Value),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Event {
    pub id: usize,
    #[serde(rename = "type")]
    pub event_type: EventType,
    //content: Value,
}

impl Event {
    pub fn new(id: usize, event_type: EventType) -> Event {
        Event { id, event_type }
    }
    pub fn try_from(msg: &MessageContent) -> JuntaResult<Event> {
        match msg {
            MessageContent::Binary(b) => Ok(serde_cbor::from_slice(b)?),
            MessageContent::Text(b) => Ok(serde_json::from_str(b)?),
        }
    }

    pub fn to_binary(&self) -> JuntaResult<Vec<u8>> {
        Ok(serde_cbor::to_vec(&self)?)
    }

    pub fn to_text(&self) -> JuntaResult<String> {
        Ok(serde_json::to_string(&self)?)
    }
}
