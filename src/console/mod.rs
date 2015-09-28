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

pub struct CVar<T: Sized + 'static> {
	pub name: &'static str,
	pub ty: PhantomData<T>,
	pub description: &'static str,
	pub mutable: bool,
	pub serializable: bool,	
	pub default: &'static Fn() -> T,
}

impl <T> CVar<T> {

}

impl Var for CVar<String> {}

pub trait Var {

}

pub struct Console {
	vars: Vec<Box<Var>>,
}

impl Console {
	pub fn new() -> Console {
		Console {
			vars: Vec::new(),
		}
	}

	pub fn register<T>(&mut self, var: CVar<T>) where CVar<T> : Var {
		self.vars.push(Box::new(var));
	}
}