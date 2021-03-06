use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

use crate::{MissingRefsError, Resolvable, Resolver};

pub type Object = serde_json::Map<String, serde_json::Value>;
pub type Record<T> = TypedRecord<T>;

/// An error that occurs while encoding a record.
#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Serialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serialization did not return an object")]
    NotAnObject,
}

/// An error that occurs while decoding a record.
#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Deserialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Type mismatch: expected {0}, got {1}")]
    TypeMismatch(String, String),
    #[error("Deserialization did not return an object")]
    NotAnObject,
}

/// Record metadata.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RecordMeta {
    guid: String,
    #[serde(rename = "type")]
    typ: String,
    id: String,
    source: String,
    seq: u32,
    version: u32,
    timestamp: u32,
}

/// A trait to implement on value structs for typed [Record]s.
pub trait TypedValue: fmt::Debug + Any + Serialize + DeserializeOwned + std::clone::Clone {
    /// A string to uniquely identify this record type.
    const NAME: &'static str;

    /// Get a human-readable label for this record.
    ///
    /// This method is optional and returns None by default. Record types may implement this method
    /// to return a title or headline.
    fn label(&self) -> Option<&'_ str> {
        None
    }

    /// Get the guid string for this record type and an id string.
    fn guid(id: &str) -> String {
        format!("{}_{}", Self::NAME, id)
    }
}

/// An untyped record is a record without static typing. The value is encoded as a JSON object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UntypedRecord {
    #[serde(rename = "$meta")]
    meta: RecordMeta,
    #[serde(flatten)]
    value: Object,
}

impl UntypedRecord {
    /// Create an untyped record from type and id strings and a [serde_json::Value].
    ///
    /// The value should be a [serde_json::Map], otherwise this method will return an
    /// [DecodingError::NotAnObject].
    pub fn with_typ_id_value(typ: &str, id: &str, value: Value) -> Result<Self, DecodingError> {
        let meta = RecordMeta {
            typ: typ.into(),
            id: id.into(),
            ..Default::default()
        };
        let value = match value {
            Value::Object(object) => object,
            _ => return Err(DecodingError::NotAnObject),
        };
        Ok(Self { meta, value })
    }

    /// Convert this untyped record into a typed [Record].
    pub fn into_typed_record<T: TypedValue + DeserializeOwned + Clone + 'static>(
        self,
    ) -> Result<TypedRecord<T>, DecodingError> {
        if self.meta.typ.as_str() != T::NAME {
            return Err(DecodingError::TypeMismatch(
                T::NAME.to_string(),
                self.meta.typ.clone(),
            ));
        }
        let value: T = serde_json::from_value(Value::Object(self.value))?;
        let record = TypedRecord {
            meta: self.meta,
            value,
        };
        Ok(record)
    }

    /// Convert the untyped record into a JSON [Object].
    pub fn into_json_object(self) -> Result<Object, EncodingError> {
        let value = serde_json::to_value(self)?;
        if let Value::Object(value) = value {
            Ok(value)
        } else {
            Err(EncodingError::NotAnObject)
        }
    }

    /// Get the guid of the record.
    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    /// Get the id of the record.
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get the type of the record.
    pub fn typ(&self) -> &str {
        &self.meta.typ
    }

    /// Merge this record's value with another JSON value.
    pub fn merge_json_value(
        &mut self,
        value_to_merge: serde_json::Value,
    ) -> Result<(), EncodingError> {
        // TODO: Get rid of this clone?
        let mut value = Value::Object(self.value.clone());
        json_patch::merge(&mut value, &value_to_merge);
        // TODO: Validate the result?
        match value {
            Value::Object(value) => {
                self.value = value;
                Ok(())
            }
            _ => Err(EncodingError::NotAnObject),
        }
    }
}

impl<T> TryFrom<UntypedRecord> for TypedRecord<T>
where
    T: TypedValue,
{
    type Error = DecodingError;
    fn try_from(record: UntypedRecord) -> Result<Self, Self::Error> {
        record.into_typed_record()
    }
}

impl<T> TryFrom<TypedRecord<T>> for UntypedRecord
where
    T: TypedValue,
{
    type Error = EncodingError;
    fn try_from(record: TypedRecord<T>) -> Result<Self, Self::Error> {
        record.into_untyped_record()
    }
}

/// A record with a strongly typed value.
///
/// All values should implement [TypedValue].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypedRecord<T>
where
    T: Clone,
{
    #[serde(rename = "$meta")]
    pub meta: RecordMeta,
    #[serde(flatten)]
    pub value: T,
}

impl<T> TypedRecord<T>
where
    T: TypedValue,
{
    /// Get the guid of the record.
    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    /// Get the id of the record.
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get the typ of the record.
    pub fn typ(&self) -> &str {
        &self.meta.typ
    }

    /// Create a new record from an id and a value.
    pub fn from_id_and_value(id: impl ToString, value: T) -> Self {
        let id = id.to_string();
        let typ = T::NAME.to_string();
        let guid = format!("{}_{}", typ, id);
        let meta = RecordMeta {
            guid,
            id,
            typ,
            ..Default::default()
        };
        Self { meta, value }
    }

    /// Convert this record into an [UntypedRecord].
    ///
    /// This can be unwrapped by default as it only fails if the record value would not serialize
    /// to an object (which should be treated as a bug).
    pub fn into_untyped_record(self) -> Result<UntypedRecord, EncodingError>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(self.value)?;
        let value = if let Value::Object(value) = value {
            value
        } else {
            return Err(EncodingError::NotAnObject);
        };
        let record = UntypedRecord {
            meta: self.meta,
            value,
        };
        Ok(record)
    }

    /// Convert this record into a JSON [Object].
    pub fn into_json_object(self) -> Result<Object, EncodingError> {
        let value = serde_json::to_value(self)?;
        if let Value::Object(value) = value {
            Ok(value)
        } else {
            Err(EncodingError::NotAnObject)
        }
    }
}

impl<T> TypedRecord<T>
where
    T: Resolvable + Send,
{
    /// Resolve all references within this record into loaded records.
    pub async fn resolve_refs<R: Resolver + Send + Sync>(
        &mut self,
        resolver: &R,
    ) -> Result<(), MissingRefsError> {
        self.value.resolve_refs(resolver).await
    }

    /// Extract all loaded referenced records from within the record, converting the references
    /// back to IDs.
    pub fn extract_refs(&mut self) -> Vec<UntypedRecord> {
        self.value.extract_refs()
    }
}

// impl<T> TypedRecord<T>
// where
//     T: TypedValue,
// {
//     fn downcast<T: 'static>(&self) -> Option<&T> {
//         let value: &dyn Any = &self.value;
//         if let Some(value) = value.downcast_ref::<T>() {
//             Some(value)
//         } else {
//             None
//         }
//     }

//     fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
//         let value: &mut dyn Any = &mut self.value;
//         if let Some(mut value) = value.downcast_mut::<T>() {
//             Some(value)
//         } else {
//             None
//         }
//     }

//     fn into_downcast<T>(self) -> Option<T> {
//         // let value: Box<dyn Any> = self.value.downcast();
//         downcast_box::<T>(self.value)
//         // let value = *self.value;
//         // let value: dyn Any = self.value;
//         // let value: Box<dyn Any> = self.value;
//         // if let Ok(value) = Box<Any>::downcast::<T>(self.value) {
//         //     Some(*value)
//         // } else {
//         //     None
//         // }
//         // None
//     }

//     fn downcast_box<T>(value: Box<dyn Any>) -> Option<T>
//     where
//         T: 'static,
//     {
//         if let Ok(value) = value.downcast::<T>() {
//             Some(*value)
//         } else {
//             None
//         }
//     }
// }
