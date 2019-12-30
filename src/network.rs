use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// NetworkCommandType is type of command sent between client and server
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ClientRequestType {
    /// Get retrives key, value pair
    Get,
    /// Set inserts key, value pair
    Set,
    /// Rm removes key, value pair
    Rm,
}

/// NetworkCommand is command sent of TCP between client and server.
#[derive(Serialize, Debug, PartialEq)]
pub struct ClientRequest {
    /// command_type is type of client request: Get, Set, Rm
    pub command_type: ClientRequestType,
    /// key is required
    pub key: String,
    /// value is optional
    pub value: String,
}

impl<'de> Deserialize<'de> for ClientRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            CommandType,
            Key,
            Value,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`command_type`, `key`, or `value`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "command_type" => Ok(Field::CommandType),
                            "key" => Ok(Field::Key),
                            "value" => Ok(Field::Value),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
        struct ClientRequestVisitor;

        impl<'de> Visitor<'de> for ClientRequestVisitor {
            type Value = ClientRequest;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ClientRequest")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ClientRequest, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let command_type = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let key = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let value = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(ClientRequest {
                    command_type,
                    key,
                    value,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<ClientRequest, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut command_type = None;
                let mut key = None;
                let mut value = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        Field::CommandType => {
                            if command_type.is_some() {
                                return Err(de::Error::duplicate_field("command_type"));
                            }
                            command_type = Some(map.next_value()?);
                        }
                        Field::Key => {
                            if key.is_some() {
                                return Err(de::Error::duplicate_field("key"));
                            }
                            key = Some(map.next_value()?);
                        }
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                    }
                }
                let command_type =
                    command_type.ok_or_else(|| de::Error::missing_field("command_type"))?;
                let key = key.ok_or_else(|| de::Error::missing_field("key"))?;
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                Ok(ClientRequest {
                    command_type,
                    key,
                    value,
                })
            }
        }
        const FIELDS: &'static [&'static str] = &["command_type", "key", "value"];
        deserializer.deserialize_struct("ClientRequest", FIELDS, ClientRequestVisitor)
    }
}

/// Response is used to respond with OK or value
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Response {
    /// arbitrary string value
    pub value: String,
    /// error message
    pub error: String,
}
