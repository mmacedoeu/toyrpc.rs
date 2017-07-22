// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Error, Visitor, MapAccess};
use serde::de::value::MapAccessDeserializer;
use serde_json::Value;
use serde_json::de::from_reader;
use serde_json::ser::to_string;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub struct UserDefaults {
	pub is_first_launch: bool,
	pub tracing: bool,
	pub fat_db: bool,
}

impl Serialize for UserDefaults {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer
	{
		let mut map: BTreeMap<String, Value> = BTreeMap::new();
		map.insert("tracing".into(), Value::Bool(self.tracing));
		map.insert("fat_db".into(), Value::Bool(self.fat_db));
		map.serialize(serializer)
	}
}

struct UserDefaultsVisitor;

impl<'a> Deserialize<'a> for UserDefaults {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		deserializer.deserialize_any(UserDefaultsVisitor)
	}
}

impl<'a> Visitor<'a> for UserDefaultsVisitor {
	type Value = UserDefaults;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid UserDefaults object")
	}

	fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
		where V: MapAccess<'a>
	{
		{
			let mut map: BTreeMap<String, Value> = Deserialize::deserialize(MapAccessDeserializer::new(visitor))?;
			let tracing: Value = map.remove("tracing").ok_or_else(|| Error::custom("missing tracing"))?;
			let tracing = tracing.as_bool().ok_or_else(|| Error::custom("invalid tracing value"))?;
			let fat_db: Value = map.remove("fat_db").unwrap_or_else(|| Value::Bool(false));
			let fat_db = fat_db.as_bool().ok_or_else(|| Error::custom("invalid fat_db value"))?;

			let mode: Value = map.remove("mode").unwrap_or_else(|| Value::String("active".to_owned()));

			let user_defaults = UserDefaults {
				is_first_launch: false,
				tracing: tracing,
				fat_db: fat_db,
			};

			Ok(user_defaults)
		}
	}
}

impl Default for UserDefaults {
	fn default() -> Self {
		UserDefaults {
			is_first_launch: true,
			tracing: false,
			fat_db: false,
		}
	}
}

impl UserDefaults {
	pub fn load<P>(path: P) -> Result<Self, String>
		where P: AsRef<Path>
	{
		match File::open(path) {
			Ok(file) => {
				match from_reader(file) {
					Ok(defaults) => Ok(defaults),
					Err(e) => {
						warn!("Error loading user defaults file: {:?}", e);
						Ok(UserDefaults::default())
					}
				}
			}
			_ => Ok(UserDefaults::default()),
		}
	}

	pub fn save<P>(&self, path: P) -> Result<(), String>
		where P: AsRef<Path>
	{
		let mut file: File = File::create(path).map_err(|_| "Cannot create user defaults file".to_owned())?;
		file.write_all(to_string(&self).unwrap().as_bytes())
		    .map_err(|_| "Failed to save user defaults".to_owned())
	}
}
