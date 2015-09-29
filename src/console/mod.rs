// Copyright 2015 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::marker::PhantomData;
use std::collections::HashMap;
use std::any::Any;

pub struct CVar<T: Sized + Any + 'static> {
	pub name: &'static str,
	pub ty: PhantomData<T>,
	pub description: &'static str,
	pub mutable: bool,
	pub serializable: bool,	
	pub default: &'static Fn() -> T,
}

impl Var for CVar<String> {}

pub trait Var {

}

pub struct Console {
	vars: HashMap<&'static str, Box<Var>>,
	var_values: HashMap<&'static str, Box<Any>>,
}

impl Console {
	pub fn new() -> Console {
		Console {
			vars: HashMap::new(),
			var_values: HashMap::new(),
		}
	}

	pub fn register<T: Sized + Any>(&mut self, var: CVar<T>) where CVar<T> : Var {
		if self.vars.contains_key(var.name) {
			panic!("Key registered twice {}", var.name);
		}
		self.var_values.insert(var.name, Box::new((var.default)()));
		self.vars.insert(var.name, Box::new(var));
	}

	pub fn get<T: Sized + Any>(&self, var: CVar<T>) -> &T where CVar<T> : Var {
		// Should never fail
		self.var_values.get(var.name).unwrap().downcast_ref::<T>().unwrap()
	}

	pub fn set<T: Sized + Any>(&mut self, var: CVar<T>, val: T) where CVar<T> : Var {
		self.var_values.insert(var.name, Box::new(val));
	}
}