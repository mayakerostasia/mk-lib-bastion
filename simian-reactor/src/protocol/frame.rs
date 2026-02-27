use super::proc::Proc;
use super::SimianFrame;
use bytes::Bytes;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use rkyv::rancor::Error as RankError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::From;
use tower::BoxError;

/// Wrapper for serde_json::Value to enable rkyv serialization
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct JsonValue(pub String);

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        JsonValue(value.to_string())
    }
}

impl From<JsonValue> for Value {
    fn from(value: JsonValue) -> Self {
        serde_json::from_str(&value.0).unwrap_or(Value::Null)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Frame {
    Ping,
    Pong,
    Msg(String),
    Bytes(Box<[u8]>),
    SendBox(SendBox),
    Json(JsonValue),
    Close,
    Exec(Proc),
    Fin,
    Error(String),
}
impl SimianFrame for Frame {}

impl From<Frame> for Bytes {
    fn from(value: Frame) -> Self {
        match value.encode() {
            Ok(frame) => frame.into(),
            Err(e) => Frame::Error(e.to_string()).into(),
        }
    }
}

impl From<Bytes> for Frame {
    fn from(value: Bytes) -> Self {
        match Frame::decode(&value) {
            Ok(frame) => frame,
            Err(e) => Frame::Error(e.to_string()),
        }
    }
}

impl Frame {
    pub fn json(json: Value) -> Frame {
        Frame::Json(json.into())
    }

    pub fn message(msg: &str) -> Frame {
        Frame::Msg(msg.to_string())
    }

    pub fn bytes(data: Box<[u8]>) -> Frame {
        Frame::Bytes(data)
    }

    pub fn exec(cmd: &str, args: Vec<&str>) -> Frame {
        Frame::Exec(Proc::new(cmd, args))
    }

    pub fn ping() -> Frame {
        Frame::Ping
    }

    pub fn pong() -> Frame {
        Frame::Pong
    }
    
    /// Extract JSON value from Frame::Json variant
    pub fn as_json(&self) -> Option<Value> {
        match self {
            Frame::Json(json_value) => Some(json_value.clone().into()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SendBox {
    pub from: String,
    pub addr: String,
    pub data: Box<[u8]>,
}

impl Frame {
    /// Encode Frame to bytes using rkyv
    pub fn encode(&self) -> Result<Vec<u8>, BoxError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
        Ok(bytes.to_vec())
    }

    /// Decode Frame from bytes using rkyv
    pub fn decode(data: &[u8]) -> Result<Self, BoxError> {
        let frame: Frame = rkyv::from_bytes::<Frame, RankError>(&data)?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tower::BoxError;

    #[test]
    fn test_json_serialize() -> Result<(), BoxError> {
        let json = json!({"hello": "world"});
        let fram = Frame::json(json);
        let encoded = fram.encode()?;
        eprintln!("Encoded: {:?}", encoded);
        Ok(())
    }
    
    #[test]
    fn test_frame_roundtrip() -> Result<(), BoxError> {
        // Test all frame variants
        let frames = vec![
            Frame::Ping,
            Frame::Pong,
            Frame::Msg("test message".to_string()),
            Frame::Bytes(vec![1, 2, 3, 4].into_boxed_slice()),
            Frame::json(json!({"key": "value", "number": 42})),
            Frame::Close,
            Frame::Exec(Proc::new("test", vec!["arg1", "arg2"])),
            Frame::Fin,
            Frame::Error("test error".to_string()),
        ];
        
        for frame in frames {
            let encoded = frame.encode()?;
            let decoded = Frame::decode(&encoded)?;
            assert_eq!(frame, decoded, "Roundtrip failed for {:?}", frame);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_frame_from_bytes() -> Result<(), BoxError> {
        let original = Frame::json(json!({"test": "data"}));
        let bytes: Bytes = original.clone().into();
        let decoded: Frame = Frame::from(bytes);
        assert_eq!(original, decoded);
        Ok(())
    }
    
    #[test]
    fn test_sendbox_roundtrip() -> Result<(), BoxError> {
        let sendbox = SendBox {
            from: "sender".to_string(),
            addr: "recipient".to_string(),
            data: vec![10, 20, 30].into_boxed_slice(),
        };
        let frame = Frame::SendBox(sendbox.clone());
        
        let encoded = frame.encode()?;
        let decoded = Frame::decode(&encoded)?;
        
        if let Frame::SendBox(decoded_box) = decoded {
            assert_eq!(sendbox, decoded_box);
        } else {
            panic!("Expected Frame::SendBox");
        }
        
        Ok(())
    }
    
    #[test]
    fn test_json_value_conversion() {
        let json_val = json!({"nested": {"key": "value"}});
        let frame = Frame::json(json_val.clone());
        let extracted = frame.as_json().unwrap();
        assert_eq!(json_val, extracted);
    }
}
