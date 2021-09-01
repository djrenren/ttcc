use serde::{de, Serialize, Deserialize};
use serde_json::{Value, from_str, from_reader, to_value};
use strong_xml::XmlRead;

pub mod roll;
use roll::RollExpr;

#[derive(Clone, Debug, Default, XmlRead)]
#[xml(tag="feature")]
pub struct Feature {
    #[xml(attr="id")]
    pub id: String,
    #[xml(flatten_text="meta")]
    pub tags: Vec<String>,
    #[xml(child="data", child = "add", child="choice", child="ref", child="roll")]
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, Default, XmlRead)]
#[xml(tag="library")]
pub struct Library {
    #[xml(child="feature")]
    pub features: Vec<Feature>,
}

#[derive(Clone, Debug, XmlRead)]
pub enum Trait {
    #[xml(tag="data")]
    Data {
        #[xml(attr="name")]
        name: String,
        #[xml(text)]
        value: Value,
    },
    #[xml(tag="add")]
    Add {
        #[xml(attr="value")]
        name: String,
        #[xml(text)]
        value: Value,
    },
    #[xml(tag="choice")]
    Choice {
        #[xml(attr="id")]
        id: String,
        #[xml(attr="default")]
        default: Option<String>,
        #[xml(child="and", child = "or", child="meta")]
        query: Query,

    },
    #[xml(tag="ref")]
    Ref {
        #[xml(attr="id")]
        id: String,
    },
    #[xml(tag="roll")]
    Roll {
        #[xml(attr="name")]
        name: String,
        #[xml(attr="expr")]
        expr: RollExpr
    }
}




#[derive(Clone, Debug, XmlRead)]
pub enum Query {
    #[xml(tag="meta")]
    Meta(Meta),
    #[xml(tag="and")]
    And(And),
    #[xml(tag="or")]
    Or(Or)
}

#[derive(Clone, Debug, XmlRead)]
#[xml(tag="meta")]
pub struct Meta {
    #[xml(text)]
    pub tag: String
}

#[derive(Clone, Debug, XmlRead)]
#[xml(tag="and")]
pub struct And {
    #[xml(child="or", child="and", child="meta")]
    pub queries: Vec<Query>
}

#[derive(Clone, Debug, XmlRead)]
#[xml(tag="or")]
pub struct Or {
    #[xml(child="meta")]
    pub queries: Vec<Meta>
}
