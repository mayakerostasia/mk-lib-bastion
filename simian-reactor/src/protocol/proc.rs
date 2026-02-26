use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Proc {
    pub cmd: String,
    pub args: Vec<String>,
}

impl Proc {
    pub fn new(cmd: &str, args: Vec<&str>) -> Proc {
        Proc {
            cmd: cmd.to_string(),
            args: args.iter().map(|t| t.to_string()).collect(),
        }
    }
}
