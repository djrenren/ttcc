use feature_db::{FeatureDB, MemDB};
use serde_json::Value;
use std::collections::{HashMap, BTreeSet};

mod material;
use material::{Feature, Trait, roll::Roll};

mod feature_db;

use crate::material::Library;
struct Creation<'c> {
    db: &'c dyn FeatureDB,
    features: Vec<Feature>,
    choices: HashMap<String, String>,
}

enum Event {
    Feature(Feature),
    Choice(String, HashMap<String, Feature>),
}

impl<'l> Creation<'l> {
    fn new(db: &'l MemDB) -> Self {
        Self {
            db,
            features: vec![],
            choices: HashMap::new(),
        }
    }
}

#[derive(Default, Debug)]
struct Character {
    features: BTreeSet<String>,
    choices: HashMap<String, Vec<String>>,
    values: HashMap<String, Value>,
    rolls: HashMap<String, Roll>
}

impl<'f> Creation<'f> {
    fn adopt_feature(&mut self, name: &str) {
        self.features.push(self.db.lookup_feature(name).unwrap());
    }

    fn make_choice(&mut self, name: &str, choice: &str) {
        self.choices.insert(name.to_string(), choice.to_string());
    }

    fn eval(&self) -> Character {
        let mut c = Character::default();
        for f in &self.features {
            self.eval_feature(f, &mut c)
        }
        c
    }

    fn eval_feature(&self, f: &Feature, c: &mut Character) {
        c.features.insert(f.id.clone());
        for t in &f.traits {
            match t {
                Trait::Data { name, value } => {
                    c.values.insert(name.clone(), value.clone());
                }
                Trait::Add { name, value } => {
                    c.values.insert(
                        name.clone(),
                        (c.values[name].as_i64().unwrap() + value.as_i64().unwrap()).into(),
                    );
                }
                Trait::Choice { id, query , default} => {
                    let options = self.db.query(&query);
                    c.choices
                        .insert(id.clone(), options.iter().map(|f| f.id.clone()).collect());
                    let maybe_feature = self
                        .choices
                        .get(id)
                        .or(default.as_ref())
                        .and_then(|id| options.iter().find(|f| &f.id == id));

                    if let Some(f) = maybe_feature {
                        self.eval_feature(f, c);
                    }
                }
                Trait::Ref { id } => self.eval_feature(&self.db.lookup_feature(&id).unwrap(), c),
                Trait::Roll { name, expr} => {
                    let roll = c.rolls.entry(name.clone()).or_insert(expr.roll());
                    c.values.insert(name.clone(), roll.value().into());
                }
            }
        }
    }
}

#[test]
fn parse() {
    use strong_xml::XmlRead;

    let library = Library::from_str(&std::fs::read_to_string("data/pf2_srd.xml").unwrap()).unwrap();
    let db: MemDB = library.into();

    let mut creation = Creation::new(&db);
    creation.adopt_feature("pathfinder");
    creation.make_choice("ancestry", "ancestry.dwarf");
    creation.make_choice("free-attr", "boost.str");
    creation.make_choice("free-attr", "boost.str");

    let character = creation.eval();
    eprintln!("{:#?}", character);
    assert!(character.features.contains("attrs.default"));
    assert_eq!(character.values["attr.con"].as_i64().unwrap(), 12);
    assert_eq!(character.values["attr.str"].as_i64().unwrap(), 12);
    assert_eq!(character.values["attr.cha"].as_i64().unwrap(), 8);
}
