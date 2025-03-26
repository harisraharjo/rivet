//taken from https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
use rustc_hash::FxHashMap;
use std::mem;

#[derive(Debug)]
pub struct Interner {
    map: FxHashMap<&'static str, u32>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}
impl Interner {
    pub fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        Interner {
            map: FxHashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    pub fn intern(&mut self, name: &str) -> StrId {
        if let Some(&id) = self.map.get(name) {
            return StrId(id);
        }
        let name = unsafe { self.alloc(name) };
        let id = self.map.len() as u32;
        self.map.insert(name, id);
        let id = StrId(id);
        self.vec.push(name);
        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);
        id
    }

    pub fn lookup(&self, id: StrId) -> &str {
        self.vec[usize::from(id)]
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }
        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };
        unsafe { &*(interned as *const str) }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StrId(u32);

impl From<StrId> for usize {
    fn from(value: StrId) -> Self {
        value.0 as usize
    }
}

// use std::collections::HashMap;

// pub struct InternerO<'a> {
//     map: std::collections::HashMap<&'a str, u32>,
//     vec: Vec<&'a str>,
//     arena: &'a Slab<u8>,
// }

// impl InternerO<'_> {
//     fn new(arena: &Slab<u8>) -> InternerO {
//         InternerO {
//             map: std::collections::HashMap::new(),
//             vec: Vec::new(),
//             arena,
//         }
//     }

//     pub fn intern(&mut self, name: &str) -> u32 {
//         if let Some(&idx) = self.map.get(name) {
//             return idx;
//         }
//         let idx = self.vec.len() as u32;
//         let arena_id = self.arena.insert(name);
//         let name = self.arena[arena_id];
//         self.map.insert(name, idx);
//         self.vec.push(name);

//         debug_assert!(self.lookup(idx) == name);
//         debug_assert!(self.intern(name) == idx);

//         idx
//     }

//     pub fn lookup(&self, idx: u32) -> &str {
//         self.vec[idx as usize]
//     }
// }
