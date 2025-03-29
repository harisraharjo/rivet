use bumpalo::Bump;
//taken from https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
use rustc_hash::FxHashMap;
use std::mem;

#[derive(Debug)]
pub struct Interner {
    map: FxHashMap<&'static str, StrId>,
    vec: Vec<&'static str>,
    // buf: String,
    // full: Vec<String>,
    arena: Bump,
}
impl Interner {
    pub fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        Interner {
            map: FxHashMap::default(),
            vec: Vec::new(),
            // buf: String::with_capacity(cap),
            // full: Vec::new(),
            arena: Bump::with_capacity(cap),
        }
    }

    pub fn intern(&mut self, name: &str) -> StrId {
        if let Some(id) = self.map.get(name) {
            return *id;
        }
        let id = self.generate_id();
        self.insert(name, id);
        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);
        id
    }

    pub fn insert(&mut self, name: &str, v: StrId) {
        let name = unsafe { self.alloc(name) };
        self.map.insert(name, v);
        self.vec.push(name);
    }

    pub fn lookup(&self, id: StrId) -> &str {
        self.vec[usize::from(id)]
    }

    ///Generate the next `id`
    pub fn generate_id(&self) -> StrId {
        StrId(self.map.len() as u32)
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        // let cap = self.buf.capacity();
        // if cap < self.buf.len() + name.len() {
        //     let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
        //     let new_buf = String::with_capacity(new_cap);
        //     let old_buf = mem::replace(&mut self.buf, new_buf);
        //     self.full.push(old_buf);
        // }
        // let interned = {
        //     let start = self.buf.len();
        //     self.buf.push_str(name);
        //     &self.buf[start..]
        // };
        // unsafe { &*(interned as *const str) }

        let allocated = self.arena.alloc_str(name);
        // Safety: The arena lives as long as the interner, which persists for the
        // program duration, making the 'static cast safe here.
        unsafe { &*(allocated as *const str) as &'static str }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct StrId(u32);

impl StrId {
    pub fn set(&mut self, value: u32) {
        self.0 = value;
    }
}

impl From<StrId> for usize {
    fn from(value: StrId) -> Self {
        value.0 as usize
    }
}

impl From<StrId> for u32 {
    fn from(value: StrId) -> Self {
        value.0
    }
}

// use bumpalo::Bump;
// // use std::collections::HashSet;

// /// A production-grade string interner that stores unique strings and returns references to them.
// pub struct StringInterner {
//     arena: Bump,
//     // set: HashSet<&'static str>,
//     map: FxHashMap<&'static str, StrId>,
//     vec: Vec<&'static str>,
// }

// impl StringInterner {
//     /// Creates a new string interner.
//     pub fn new() -> Self {
//         StringInterner {
//             arena: Bump::new(),
//             map: FxHashMap::default(),
//             vec: Vec::new(),
//         }
//     }

//     /// Interns a string and returns a reference to the unique instance.
//     ///
//     /// If the string already exists, returns the existing reference.
//     /// If not, allocates it in the arena, stores it in the map, and returns the reference.
//     pub fn intern(&mut self, s: &str) -> StrId {
//         if let Some(&interned) = self.map.get(s) {
//             interned
//         } else {
//             // Allocate the string in the arena
//             // let allocated = self.arena.alloc_str(s);
//             // let name = unsafe { self.alloc(name) };
//             // Safety: The arena lives as long as the interner, and we assume the interner
//             // persists for the program duration, making the 'static cast safe in this context.
//             let interned = unsafe { &*(s as *const str) };
//             // let interned = unsafe { &*(allocated as *const str) as &'static str };
//             let str_id = self.generate_id();
//             self.map.insert(interned, str_id);
//             self.vec.push(name);
//             str_id
//         }
//     }

//     ///Generate the next `id`
//     pub fn generate_id(&self) -> StrId {
//         StrId(self.map.len() as u32)
//     }
// }

// impl Default for StringInterner {
//     fn default() -> Self {
//         Self::new()
//     }
// }
