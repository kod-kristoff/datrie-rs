use core::slice;

pub type TrieChar = u8;

#[derive(Debug, Clone)]
pub struct TrieCharString {
    inner: Vec<TrieChar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NulError(usize, Vec<TrieChar>);

impl TrieCharString {
    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(0);
    }
    pub fn new<T: Into<Vec<TrieChar>>>(t: T) -> Result<TrieCharString, NulError> {
        let bytes = t.into();

        match memchr::memchr(0, &bytes) {
            Some(i) => Err(NulError(i, bytes)),
            None => Ok(Self::_from_vec_unchecked(bytes)),
        }
    }

    fn _from_vec_unchecked(mut v: Vec<TrieChar>) -> TrieCharString {
        v.reserve_exact(1);
        v.push(0);
        Self { inner: v }
    }
    pub unsafe fn strdup(&mut self, str: *const u8, len: usize) {
        assert!(self.inner.len() >= len);
        let mut p = str;
        for i in 0..len {
            self.inner[i] = *p;
            p = p.offset(1);
        }
        assert_eq!(self.inner[self.inner.len() - 1], 0);
    }

    pub unsafe fn replace_from_ptr(&mut self, str: *const TrieChar) {
        self.inner.clear();
        if !str.is_null() {
            let mut p = str;
            loop {
                self.inner.push(*p);
                if *p == 0 {
                    break;
                }
                p = p.offset(1);
            }
        } else {
            self.inner.push(9);
        }
    }

    pub fn as_ptr(&self) -> *const TrieChar {
        self.inner.as_ptr()
    }

    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[TrieChar] {
        // SAFETY: CString has a length at least 1
        unsafe { self.inner.get_unchecked(..self.inner.len() - 1) }
    }

    pub fn len(&self) -> usize {
        self.inner.len() - 1
    }
}

impl Default for TrieCharString {
    fn default() -> Self {
        Self { inner: vec![0; 1] }
    }
}

// impl From<Vec<u8>> for TrieString {
//     fn from(value: Vec<u8>) -> Self {
//         Self { inner: value }
//     }
// }
// #[repr(C)]
// pub struct TrieStr {
//     inner: [TrieChar],
// }

// impl TrieStr {
//     pub unsafe fn from_ptr<'a>(ptr: *const TrieChar) -> &'a TrieStr {
//         let len = unsafe { gen_strlen(ptr) };

//         unsafe { Self::from_slice_with_nul_unchecked(slice::from_raw_parts(ptr, len)) }
//     }

//     pub unsafe fn from_slice_with_nul_unchecked(t: &[TrieChar]) -> &TrieStr {
//         debug_assert!(!t.is_empty() && t[t.len() - 1] == 0);

//         unsafe { &*(t as *const TrieChar as *const TrieStr) }
//     }
// }

// unsafe fn gen_strlen(s: *const TrieChar) -> usize {
//     debug_assert!(!s.is_null());
//     let mut p = s;
//     let mut len = 0;
//     loop {
//         p = p.offset(1);
//         if *p == 0 {
//             break;
//         }
//         len += 1;
//     }
//     len
// }
