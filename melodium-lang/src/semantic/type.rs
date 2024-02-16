//! Module for Type identification and structure semantic analysis.

use super::{DeclarativeElement, DeclarativeElementType, Node, Reference, Use};
use crate::text::PositionnedString;
use crate::{text::Type as TextType, ScriptResult};
use crate::{Path, ScriptError};
use core::slice::Iter;
use melodium_common::descriptor::{
    Collection, DataType as DataTypeDescriptor, DescribedType as DescribedTypeDescriptor, Entry,
    Flow as FlowDescriptor, Generic,
};
use std::fmt;
use std::sync::{Arc, RwLock, Weak};

/// Enum for type flow identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeFlow {
    /// Data flow is blocking.
    Block,
    /// Data flow is a stream.
    Stream,
}

impl fmt::Display for TypeFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeFlow::Block => "Block",
                TypeFlow::Stream => "Stream",
            }
        )
    }
}

#[derive(Debug, Clone)]
enum RefersTo {
    // Designates either core type or generic one
    Implicit(Reference<()>),
    // Designates data type imported through `use`
    Use(Reference<Use>),
}

/// Enum for type identification.
#[derive(Clone, Debug)]
enum TypeContent {
    Void,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    F32,
    F64,

    Bool,
    Byte,
    Char,
    String,

    Option(Arc<RwLock<TypeContent>>),
    Vec(Arc<RwLock<TypeContent>>),

    Other((Weak<RwLock<dyn DeclarativeElement>>, RefersTo)),
}

impl TypeContent {
    fn from_positionned_strings(
        list: &Vec<PositionnedString>,
        scope: Weak<RwLock<dyn DeclarativeElement>>,
    ) -> Result<Arc<RwLock<Self>>, ()> {
        Self::from_positionned_string(list.iter(), scope).ok_or(())
    }

    fn from_positionned_string(
        mut iter: Iter<PositionnedString>,
        scope: Weak<RwLock<dyn DeclarativeElement>>,
    ) -> Option<Arc<RwLock<Self>>> {
        let step = iter.next();
        if let Some(pos_str) = step {
            Some(Arc::new(RwLock::new(match pos_str.string.as_str() {
                "void" => Self::Void,
                "i8" => Self::I8,
                "i16" => Self::I16,
                "i32" => Self::I32,
                "i64" => Self::I64,
                "i128" => Self::I128,
                "u8" => Self::U8,
                "u16" => Self::U16,
                "u32" => Self::U32,
                "u64" => Self::U64,
                "u128" => Self::U128,
                "f32" => Self::F32,
                "f64" => Self::F64,
                "bool" => Self::Bool,
                "byte" => Self::Byte,
                "char" => Self::Char,
                "string" => Self::String,
                "Option" => Self::Option(Self::from_positionned_string(iter, scope)?),
                "Vec" => Self::Vec(Self::from_positionned_string(iter, scope)?),
                other => {
                    Self::Other((scope, RefersTo::Implicit(Reference::new(other.to_string()))))
                }
            })))
        } else {
            None
        }
    }

    fn to_descriptor(&self, collection: &Collection) -> Option<DescribedTypeDescriptor> {
        match self {
            Self::Void => Some(DescribedTypeDescriptor::Void),
            Self::I8 => Some(DescribedTypeDescriptor::I8),
            Self::I16 => Some(DescribedTypeDescriptor::I16),
            Self::I32 => Some(DescribedTypeDescriptor::I32),
            Self::I64 => Some(DescribedTypeDescriptor::I64),
            Self::I128 => Some(DescribedTypeDescriptor::I128),
            Self::U8 => Some(DescribedTypeDescriptor::U8),
            Self::U16 => Some(DescribedTypeDescriptor::U16),
            Self::U32 => Some(DescribedTypeDescriptor::U32),
            Self::U64 => Some(DescribedTypeDescriptor::U64),
            Self::U128 => Some(DescribedTypeDescriptor::U128),
            Self::F32 => Some(DescribedTypeDescriptor::F32),
            Self::F64 => Some(DescribedTypeDescriptor::F64),
            Self::Bool => Some(DescribedTypeDescriptor::Bool),
            Self::Byte => Some(DescribedTypeDescriptor::Byte),
            Self::Char => Some(DescribedTypeDescriptor::Char),
            Self::String => Some(DescribedTypeDescriptor::String),
            Self::Option(internal) => internal
                .read()
                .unwrap()
                .to_descriptor(collection)
                .map(|int| DescribedTypeDescriptor::Option(Box::new(int))),

            Self::Vec(internal) => internal
                .read()
                .unwrap()
                .to_descriptor(collection)
                .map(|int| DescribedTypeDescriptor::Vec(Box::new(int))),
            Self::Other((_, refer)) => match refer {
                RefersTo::Implicit(implicit) => Some(DescribedTypeDescriptor::Generic(Box::new(
                    Generic::new(implicit.name.clone(), Vec::new()),
                ))),
                RefersTo::Use(external) => external
                    .reference
                    .as_ref()
                    .and_then(|weak| {
                        weak.upgrade().and_then(|arc| {
                            arc.read().ok().and_then(|r#use| r#use.identifier.clone())
                        })
                    })
                    .and_then(|identifier| match collection.get(&identifier) {
                        Some(Entry::Data(data)) => {
                            Some(DescribedTypeDescriptor::Data(Box::new(Arc::clone(data))))
                        }
                        _ => None,
                    }),
            },
        }
    }

    #[allow(unused)]
    fn to_datatype(&self) -> Result<DataTypeDescriptor, ()> {
        match self {
            Self::Void => Ok(DataTypeDescriptor::Void),
            Self::I8 => Ok(DataTypeDescriptor::I8),
            Self::I16 => Ok(DataTypeDescriptor::I16),
            Self::I32 => Ok(DataTypeDescriptor::I32),
            Self::I64 => Ok(DataTypeDescriptor::I64),
            Self::I128 => Ok(DataTypeDescriptor::I128),
            Self::U8 => Ok(DataTypeDescriptor::U8),
            Self::U16 => Ok(DataTypeDescriptor::U16),
            Self::U32 => Ok(DataTypeDescriptor::U32),
            Self::U64 => Ok(DataTypeDescriptor::U64),
            Self::U128 => Ok(DataTypeDescriptor::U128),
            Self::F32 => Ok(DataTypeDescriptor::F32),
            Self::F64 => Ok(DataTypeDescriptor::F64),
            Self::Bool => Ok(DataTypeDescriptor::Bool),
            Self::Byte => Ok(DataTypeDescriptor::Byte),
            Self::Char => Ok(DataTypeDescriptor::Char),
            Self::String => Ok(DataTypeDescriptor::String),
            Self::Option(internal) => Ok(DataTypeDescriptor::Option(Box::new(
                internal.read().unwrap().to_datatype()?,
            ))),
            Self::Vec(internal) => Ok(DataTypeDescriptor::Vec(Box::new(
                internal.read().unwrap().to_datatype()?,
            ))),
            Self::Other(_) => Err(()),
        }
    }
}

impl fmt::Display for TypeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeContent::Void => write!(f, "void"),
            TypeContent::I8 => write!(f, "i8"),
            TypeContent::I16 => write!(f, "i16"),
            TypeContent::I32 => write!(f, "i32"),
            TypeContent::I64 => write!(f, "i64"),
            TypeContent::I128 => write!(f, "i128"),
            TypeContent::U8 => write!(f, "u8"),
            TypeContent::U16 => write!(f, "u16"),
            TypeContent::U32 => write!(f, "u32"),
            TypeContent::U64 => write!(f, "u64"),
            TypeContent::U128 => write!(f, "u128"),
            TypeContent::F32 => write!(f, "f32"),
            TypeContent::F64 => write!(f, "f64"),
            TypeContent::Bool => write!(f, "bool"),
            TypeContent::Byte => write!(f, "byte"),
            TypeContent::Char => write!(f, "char"),
            TypeContent::String => write!(f, "string"),
            TypeContent::Option(inner) => write!(f, "Option<{}>", inner.read().unwrap()),
            TypeContent::Vec(inner) => write!(f, "Vec<{}>", inner.read().unwrap()),
            TypeContent::Other((_, refer)) => write!(
                f,
                "{}",
                match refer {
                    RefersTo::Implicit(implicit) => &implicit.name,
                    RefersTo::Use(external) => &external.name,
                }
            ),
        }
    }
}

impl Node for TypeContent {
    fn make_references(&mut self, _path: &Path) -> ScriptResult<()> {
        match self {
            TypeContent::Other((scope, RefersTo::Implicit(reference))) => {
                let rc_script = match scope
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .declarative_element()
                {
                    DeclarativeElementType::Model(m) => m.script.upgrade().unwrap(),
                    DeclarativeElementType::Treatment(t) => t.script.upgrade().unwrap(),
                    DeclarativeElementType::None => return ScriptResult::new_success(()),
                };
                let borrowed_script = rc_script.read().unwrap();

                if let Some(r#use) = borrowed_script.find_use(&reference.name) {
                    let reference = RefersTo::Use(Reference {
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(r#use)),
                    });

                    *self = TypeContent::Other((scope.clone(), reference));
                    ScriptResult::new_success(())
                }
                // No use imported, assuming generic, that is catched after by engine design
                else {
                    ScriptResult::new_success(())
                }
            }
            _ => ScriptResult::new_success(()),
        }
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        match self {
            TypeContent::Option(inner) | TypeContent::Vec(inner) => vec![inner.clone()],
            _ => Vec::new(),
        }
    }
}

/// Structure managing and describing Type semantic analysis.
///
/// It owns the whole [text type](TextType).
///
/// Unlike most other elements of the semantic module, it does _not_ implement [Node trait](super::Node), because a type is considered as a property of its owner, not a children.
/// Also, it is a build-in element of Mélodium language so don't have any references to manage.
#[derive(Debug)]
pub struct Type {
    pub text: TextType,

    pub scope: Weak<RwLock<dyn DeclarativeElement>>,

    pub flow: TypeFlow,
    content: Arc<RwLock<TypeContent>>,
}

impl Type {
    /// Create a new semantic type, based on textual type.
    ///
    /// * `text`: the textual type.
    ///
    pub fn new(scope: Arc<RwLock<dyn DeclarativeElement>>, text: TextType) -> ScriptResult<Self> {
        let mut remove_first = false;
        let flow = text
            .level_structure
            .first()
            .map(|possible_flow| match possible_flow.string.as_str() {
                "Block" => {
                    remove_first = true;
                    TypeFlow::Block
                }
                "Stream" => {
                    remove_first = true;
                    TypeFlow::Stream
                }
                _ => TypeFlow::Block,
            })
            .unwrap_or(TypeFlow::Block);

        let mut level_structure = text.level_structure.clone();
        if remove_first {
            level_structure.remove(0);
        }
        level_structure.push(text.name.clone());

        match TypeContent::from_positionned_strings(&level_structure, Arc::downgrade(&scope))
            .map_err(|_| ScriptError::invalid_structure(177, text.name.clone()))
        {
            Ok(content) => ScriptResult::new_success(Self {
                scope: Arc::downgrade(&scope),
                content,
                text,
                flow,
            }),
            Err(error) => ScriptResult::new_failure(error),
        }
    }

    pub fn make_descriptor(
        &self,
        collection: &Collection,
    ) -> ScriptResult<(DescribedTypeDescriptor, FlowDescriptor)> {
        let flow = match self.flow {
            TypeFlow::Block => FlowDescriptor::Block,
            TypeFlow::Stream => FlowDescriptor::Stream,
        };

        if let Some(descriptor) = self.content.read().unwrap().to_descriptor(collection) {
            ScriptResult::new_success((descriptor, flow))
        } else {
            ScriptResult::new_failure(ScriptError::undeclared_data(181, self.text.name.clone()))
        }
    }
}

impl Node for Type {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        vec![Arc::clone(&self.content) as Arc<RwLock<dyn Node>>]
    }
}
