use std::any::*;
use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::hash::Hasher;
use std::ptr;
use std::mem;

fn main() {
    let mut c = Container::new();
    {
        c.put("Hello world");
        println!("{}", c.get::<&str>());
    }
    {
        c.put(55);
        println!("{}", c.get::<i32>());
    }
    println!("{}", c.get::<&str>());
    println!("{}", c.get::<i32>());

    {
        *c.get_mut::<i32>() = 82;
    }

    println!("{}", c.get::<&str>());
    println!("{}", c.get::<i32>());
}

struct TypeIdState;

impl HashState for TypeIdState {
    type Hasher = TypeIdHasher;

    fn hasher(&self) -> TypeIdHasher {
        TypeIdHasher { value: 0 }
    }
}

struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.value
    }
    fn write(&mut self, bytes: &[u8]) {
        unsafe {
            ptr::copy_nonoverlapping(&mut self.value, mem::transmute(&bytes[0]), 1)
        }
    }
}

pub struct Container {
    elems: HashMap<TypeId, Box<Any>, TypeIdState>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            elems: HashMap::with_hash_state(TypeIdState),
        }
    }

    pub fn put<T: Sized + Any>(&mut self, data: T) {
        self.elems.insert(
            TypeId::of::<T>(),
            Box::new(data)
        );
    }

    pub fn get<T: Sized + Any>(&self) -> &T {
        self.elems.get(&TypeId::of::<T>()).unwrap().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: Sized + Any>(&mut self) -> & mut T {
        self.elems.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<T>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use test;

    #[bench]
    fn bench_put(b: &mut Bencher) {
    	let mut c = Container::new();
        b.iter(|| c.put("Hello world"));
    }

    #[bench]
    fn bench_get(b: &mut Bencher) {
    	let mut c = Container::new();
    	c.put("Hello world");
    	c.put(55);
        b.iter(|| c.get::<&str>());
    }

    #[bench]
    fn bench_get_int(b: &mut Bencher) {
    	let mut c = Container::new();
    	c.put("Hello world");
    	c.put(55);
        b.iter(|| c.get::<i32>());
    }

    #[bench]
    fn bench_get_mut(b: &mut Bencher) {
    	let mut c = Container::new();
    	c.put("Hello world");
    	c.put(55);
        b.iter(|| *c.get_mut::<i32>());
    }

    #[bench]
    fn bench_alloc(b: &mut Bencher) {
        b.iter(|| test::black_box(Container::new()));
    }
}
