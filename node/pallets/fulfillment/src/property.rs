// SPDX-License-Identifier: Apache-2.0

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use serde_json::Value;
use sp_runtime::RuntimeDebug;
use sp_std::{prelude::*, str::FromStr};

pub type PropName = Vec<u8>;
pub type PropValue = Vec<u8>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct GeodeProperty {
    name: PropName,
    value: PropValue,
}

impl GeodeProperty {
    pub fn new(name: &[u8], value: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            value: value.to_vec(),
        }
    }

    pub fn name(&self) -> &[u8] {
        self.name.as_ref()
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct GeodeProperties {
    pub props: Vec<GeodeProperty>,
}

impl GeodeProperties {
    #[cfg(feature = "std")]
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        GeodeProperties::from_str(s)
    }
}

#[cfg(feature = "std")]
impl FromStr for GeodeProperties {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let json: Value = serde_json::from_str(s)?;

        let ps = match json {
            Value::Object(data) => {
                let props: Vec<GeodeProperty> = data
                    .into_iter()
                    .map(|(k, v)| {
                        GeodeProperty::new(&k.into_bytes(), v.as_str().unwrap().as_bytes())
                    })
                    .collect();
                GeodeProperties { props }
            }
            _ => unimplemented!(),
        };
        Ok(ps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_should_work() {
        let str_props = r#"
        {
            "Instance": "DC1s v2",
            "Core": "1",
            "RAM": "4 GiB",
            "Temporary storage": "50GiB"
        }
        "#;

        let mut real_props = vec![];
        real_props.push(GeodeProperty::new(b"Core", b"1"));
        real_props.push(GeodeProperty::new(b"Instance", b"DC1s v2"));
        real_props.push(GeodeProperty::new(b"RAM", b"4 GiB"));
        real_props.push(GeodeProperty::new(b"Temporary storage", b"50GiB"));

        let props = GeodeProperties { props: real_props };

        let target_props = GeodeProperties::from_json(&str_props).unwrap();
        assert_eq!(target_props, props);
    }
}
