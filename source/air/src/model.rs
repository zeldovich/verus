//! Provides an AIR-level interface to the model returned by the SMT solver
//! when it reaches a SAT conclusion

use crate::ast::{Binders, Ident, Snapshots, Typ};
use std::collections::HashMap;
use std::sync::Arc;

/// For now, expressions are just strings, but we can later change this to a more detailed enum
pub type ModelExpr = Arc<String>;

/// Represent (define-fun f (...parameters...) return-type body) from SMT model
/// (This includes constants, which have an empty parameter list.)
pub type ModelDef = Arc<ModelDefX>;
pub type ModelDefs = Arc<Vec<ModelDef>>;
#[derive(Debug)]
pub struct ModelDefX {
    pub name: Ident,
    pub params: Binders<Typ>,
    pub ret: Typ,
    pub body: ModelExpr,
}

#[derive(Debug)]
/// AIR-level model of a concrete counterexample
pub struct Model {
    /// Internal mapping of snapshot IDs to snapshots that map AIR variables to usage counts.
    /// Generated when converting mutable variables to Z3-level constants.
    id_snapshots: Snapshots,
    /// Externally facing mapping from snapshot IDs to snapshots that map AIR variables
    /// to their concrete values.
    /// TODO: Upgrade to a semantics-preserving value type, instead of String.
    /// TODO: Expose via a more abstract interface
    pub value_snapshots: HashMap<Ident, HashMap<Ident, String>>,
}

impl Model {
    /// Returns an (unpopulated) AIR model object.  Must call [build()] to fully populate.
    /// # Arguments
    /// * `model` - The model that Z3 returns
    /// * `snapshots` - Internal mapping of snapshot IDs to snapshots that map AIR variables to usage counts.
    pub fn new(snapshots: Snapshots) -> Model {
        // println!("Creating a new model with {} snapshots", snapshots.len());
        Model { id_snapshots: snapshots, value_snapshots: HashMap::new() }
    }

    pub fn translate_variable(&self, sid: &Ident, name: &Ident) -> Option<String> {
        // println!("??? {:?} {:?}", sid, name);
        let id_snapshot = &self.id_snapshots.get(sid)?;
        let var_label = id_snapshot.get(name)?;
        Some(crate::var_to_const::rename_var(name, *var_label))
    }
}
