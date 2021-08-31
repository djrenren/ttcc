use std::collections::HashMap;

use crate::material::{Feature, Library, Query};

pub trait FeatureDB {
    fn lookup_feature(&self, id: &str) -> Option<Feature>;
    fn query(&self, q: &Query) -> Vec<Feature>;
}

pub struct MemDB {
    features: HashMap<String, Feature>
}

impl FeatureDB for MemDB {
    fn lookup_feature(&self, id: &str) -> Option<Feature> {
        self.features.get(id).cloned()
    }

    fn query(&self, q: &Query) -> Vec<Feature> {
        self.query_internal(q, self.features.values()).cloned().collect()
    }

}

impl MemDB {
    fn query_internal<'a>(&self, q: &'a Query, ctx: impl Iterator<Item=&'a Feature> + 'a) -> Box<dyn Iterator<Item=&'a Feature> + 'a>{ 
        match q {
            Query::Meta(s) => Box::new(ctx.filter(move |f| f.tags.contains(&s.tag))),
            Query::And(conds) => {
                let mut opts: Box<dyn Iterator<Item=_>> = Box::new(ctx);
                for q in &conds.queries {
                    opts = self.query_internal(&q, opts);
                }
                opts
            },
            Query::Or(or) => {
                Box::new(ctx.filter(move |f| or.queries.iter().any(|m| f.tags.contains(&m.tag))))
            },
        }
    }
}

impl From<Library> for MemDB {
    fn from(l: Library) -> Self {
        Self {
            features: l.features.into_iter().map(|f| (f.id.clone(), f)).collect(),
        }
    }
}
