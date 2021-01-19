//! Code for type double representation processing.

use crate::prelude::*;

use crate::double_representation::identifier::ReferentName;
use crate::double_representation::module;

use serde::Deserialize;
use serde::Serialize;
use futures::AsyncReadExt;


// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
pub enum InvalidQualifiedName {
    #[fail(display="The qualified name is empty.")]
    EmptyName,
    #[fail(display="No module in type qualified name.")]
    NoModuleName,
}


// =====================
// === QualifiedName ===
// =====================

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
#[serde(into="String")]
#[serde(try_from="String")]
pub struct QualifiedName {
    project_name    : ReferentName,
    module_segments : Vec<ReferentName>,
    name            : String,
}

impl QualifiedName {
    pub fn from_module(module:module::QualifiedName) -> Self {
        let module::QualifiedName{project_name,id} = module;
        let mut module_segments                    = id.take_segments();
        // We may unwrap, because the `module::QualifiedName` guarantees segments to be non-empty.
        let name                                   = module_segments.pop().unwrap().into();
        QualifiedName{project_name,module_segments,name}
    }

    pub fn from_text(text:impl Str) -> FallibleResult<Self> {
        let mut all_segments    = text.as_ref().split('.');
        let project_name_str    = all_segments.next().ok_or(InvalidQualifiedName::EmptyName)?;
        let project_name        = ReferentName::new(project_name_str)?;
        let name_str            = all_segments.next_back().ok_or(InvalidQualifiedName::NoModuleName)?;
        let name                = name_str.to_owned();
        let mut module_segments = Vec::new();
        for segment in all_segments {
            module_segments.push(ReferentName::new(segment)?);
        }
        Ok(QualifiedName {project_name,module_segments,name})
    }
}


// === Conversions ===

impl TryFrom<&str> for QualifiedName {
    type Error = failure::Error;

    fn try_from(text:&str) -> Result<Self,Self::Error> {
        Self::from_text(text)
    }
}

impl TryFrom<String> for QualifiedName {
    type Error = failure::Error;

    fn try_from(text:String) -> Result<Self,Self::Error> {
        Self::from_text(text)
    }
}

impl From<module::QualifiedName> for QualifiedName {
    fn from(name:module::QualifiedName) -> Self {
        Self::from_module(name)
    }
}

impl From<QualifiedName> for String {
    fn from(name:QualifiedName) -> Self {
        String::from(&name)
    }
}

impl From<&QualifiedName> for String {
    fn from(name:&QualifiedName) -> Self {
        let project_name = std::iter::once(name.project_name.as_ref());
        let segments     = name.module_segments.iter().map(AsRef::<str>::as_ref);
        let name         = std::iter::once(name.name.as_ref());
        project_name.chain(segments).chain(name).join(",")
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = String::from(self);
        fmt::Display::fmt(&text,f)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn qualified_name_conversions() {
        unimplemented!()
    }
}