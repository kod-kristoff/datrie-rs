use core::{fmt, slice};
use std::borrow::Borrow;
use std::error::Error;
use std::ops;
use std::ptr::addr_of;

pub type TrieChar = u8;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TrieCharString {
    inner: Box<[TrieChar]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NulError(usize, Vec<TrieChar>);

/// An error indicating that a nul byte was not in the expected position.
///
/// The vector used to create a [`TrieCharString`] must have one and only one nul byte,
/// positioned at the end.
///
/// This error is created by the [`TrieCharString::from_vec_with_nul`] method.
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use datrie::trie_str::{TrieCharString, FromVecWithNulError};
///
/// let _: FromVecWithNulError = TrieCharString::from_vec_with_nul(b"f\0oo".to_vec()).unwrap_err();
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromVecWithNulError {
    error_kind: FromBytesWithNulErrorKind,
    bytes: Vec<u8>,
}

impl FromVecWithNulError {
    /// Returns a slice of [`u8`]s bytes that were attempted to convert to a [`TrieCharString`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    ///
    /// // Some invalid bytes in a vector
    /// let bytes = b"f\0oo".to_vec();
    ///
    /// let value = TrieCharString::from_vec_with_nul(bytes.clone());
    ///
    /// assert_eq!(&bytes[..], value.unwrap_err().as_bytes());
    /// ```
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns the bytes that were attempted to convert to a [`TrieCharString`].
    ///
    /// This method is carefully constructed to avoid allocation. It will
    /// consume the error, moving out the bytes, so that a copy of the bytes
    /// does not need to be made.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    ///
    /// // Some invalid bytes in a vector
    /// let bytes = b"f\0oo".to_vec();
    ///
    /// let value = TrieCharString::from_vec_with_nul(bytes.clone());
    ///
    /// assert_eq!(bytes, value.unwrap_err().into_bytes());
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
impl TrieCharString {
    pub fn new<T: Into<Vec<TrieChar>>>(t: T) -> Result<TrieCharString, NulError> {
        let bytes = t.into();

        match memchr::memchr(0, &bytes) {
            Some(i) => Err(NulError(i, bytes)),
            None => Ok(unsafe { Self::_from_vec_unchecked(bytes) }),
        }
    }

    /// Creates a C-compatible string by consuming a byte vector,
    /// without checking for interior 0 bytes.
    ///
    /// Trailing 0 byte will be appended by this function.
    ///
    /// This method is equivalent to [`TrieCharString::new`] except that no runtime
    /// assertion is made that `v` contains no 0 bytes, and it requires an
    /// actual byte vector, not anything that can be converted to one with Into.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    ///
    /// let raw = b"foo".to_vec();
    /// unsafe {
    ///     let c_string = TrieCharString::from_vec_unchecked(raw);
    /// }
    /// ```
    #[must_use]
    pub unsafe fn from_vec_unchecked(v: Vec<u8>) -> Self {
        debug_assert!(memchr::memchr(0, &v).is_none());
        unsafe { Self::_from_vec_unchecked(v) }
    }
    unsafe fn _from_vec_unchecked(mut v: Vec<TrieChar>) -> TrieCharString {
        v.reserve_exact(1);
        v.push(0);
        // dbg!(&v);
        Self {
            inner: v.into_boxed_slice(),
        }
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
    /// Extracts a [`TrieCharStr`] slice containing the entire string.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::{TrieCharString, TrieCharStr};
    ///
    /// let c_string = TrieCharString::new(b"foo".to_vec()).expect("TrieCharString::new failed");
    /// let cstr = c_string.as_trie_str();
    /// assert_eq!(cstr,
    ///            TrieCharStr::from_bytes_with_nul(b"foo\0").expect("TrieCharStr::from_bytes_with_nul failed"));
    /// ```
    #[inline]
    #[must_use]
    pub fn as_trie_str(&self) -> &TrieCharStr {
        &*self
    }

    // pub unsafe fn replace_from_ptr(&mut self, str: *const TrieChar) {
    //     self.inner.clear();
    //     if !str.is_null() {
    //         let mut p = str;
    //         loop {
    //             self.inner.push(*p);
    //             if *p == 0 {
    //                 break;
    //             }
    //             p = p.offset(1);
    //         }
    //     } else {
    //         self.inner.push(9);
    //     }
    // }
    // pub fn replace(&mut self, str: &[TrieChar]) {}

    pub fn as_ptr(&self) -> *const TrieChar {
        self.inner.as_ptr()
    }

    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[TrieChar] {
        // SAFETY: TrieCharString has a length at least 1
        unsafe { self.inner.get_unchecked(..self.inner.len() - 1) }
    }
    /// Equivalent to [`TrieCharString::as_bytes()`] except that the
    /// returned slice includes the trailing nul terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    ///
    /// let c_string = TrieCharString::new("foo").expect("TrieCharString::new failed");
    /// let bytes = c_string.as_bytes_with_nul();
    /// assert_eq!(bytes, &[b'f', b'o', b'o', b'\0']);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.inner
    }

    pub fn count_bytes(&self) -> usize {
        self.inner.len() - 1
    }

    /// Retakes ownership of a `TrieCharString` that was transferred to C via
    /// [`TrieCharString::into_raw`].
    ///
    /// Additionally, the length of the string will be recalculated from the pointer.
    ///
    /// # Safety
    ///
    /// This should only ever be called with a pointer that was earlier
    /// obtained by calling [`TrieCharString::into_raw`]. Other usage (e.g., trying to take
    /// ownership of a string that was allocated by foreign code) is likely to lead
    /// to undefined behavior or allocator corruption.
    ///
    /// It should be noted that the length isn't just "recomputed," but that
    /// the recomputed length must match the original length from the
    /// [`TrieCharString::into_raw`] call. This means the [`TrieCharString::into_raw`]/`from_raw`
    /// methods should not be used when passing the string to C functions that can
    /// modify the string's length.
    ///
    /// > **Note:** If you need to borrow a string that was allocated by
    /// > foreign code, use [`TrieCharStr`]. If you need to take ownership of
    /// > a string that was allocated by foreign code, you will need to
    /// > make your own provisions for freeing it appropriately, likely
    /// > with the foreign code's API to do that.
    ///
    /// # Examples
    ///
    /// Creates a `TrieCharString`, pass ownership to an `extern` function (via raw pointer), then retake
    /// ownership with `from_raw`:
    ///
    /// ```ignore (extern-declaration)
    /// use datrie::trie_str::TrieCharString;
    /// use std::os::raw::c_char;
    ///
    /// extern "C" {
    ///     fn some_extern_function(s: *mut c_char);
    /// }
    ///
    /// let c_string = TrieCharString::new("Hello!").expect("TrieCharString::new failed");
    /// let raw = c_string.into_raw();
    /// unsafe {
    ///     some_extern_function(raw);
    ///     let c_string = TrieCharString::from_raw(raw);
    /// }
    /// ```
    #[must_use = "call `drop(from_raw(ptr))` if you intend to drop the `TrieCharString`"]
    pub unsafe fn from_raw(ptr: *mut TrieChar) -> TrieCharString {
        // SAFETY: This is called with a pointer that was obtained from a call
        // to `TrieCharString::into_raw` and the length has not been modified. As such,
        // we know there is a NUL byte (and only one) at the end and that the
        // information about the size of the allocation is correct on Rust's
        // side.
        unsafe {
            let len = libc::strlen(ptr as *const i8) + 1; // Including the NUL byte
            let slice = slice::from_raw_parts_mut(ptr, len);

            TrieCharString {
                inner: Box::from_raw(slice as *mut [TrieChar] as *mut [u8]),
            }
        }
    }
    /// Converts a <code>[Vec]<[u8]></code> to a [`TrieCharString`] without checking the
    /// invariants on the given [`Vec`].
    ///
    /// # Safety
    ///
    /// The given [`Vec`] **must** have one nul byte as its last element.
    /// This means it cannot be empty nor have any other nul byte anywhere else.
    ///
    /// # Example
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    /// assert_eq!(
    ///     unsafe { TrieCharString::from_vec_with_nul_unchecked(b"abc\0".to_vec()) },
    ///     unsafe { TrieCharString::from_vec_unchecked(b"abc".to_vec()) }
    /// );
    /// ```
    #[must_use]
    pub unsafe fn from_vec_with_nul_unchecked(v: Vec<u8>) -> Self {
        debug_assert!(memchr::memchr(0, &v).unwrap() + 1 == v.len());
        unsafe { Self::_from_vec_with_nul_unchecked(v) }
    }

    unsafe fn _from_vec_with_nul_unchecked(v: Vec<u8>) -> Self {
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Attempts to converts a <code>[Vec]<[u8]></code> to a [`TrieCharString`].
    ///
    /// Runtime checks are present to ensure there is only one nul byte in the
    /// [`Vec`], its last element.
    ///
    /// # Errors
    ///
    /// If a nul byte is present and not the last element or no nul bytes
    /// is present, an error will be returned.
    ///
    /// # Examples
    ///
    /// A successful conversion will produce the same result as [`TrieCharString::new`]
    /// when called without the ending nul byte.
    ///
    /// ```
    /// use datrie::trie_str::TrieCharString;
    /// assert_eq!(
    ///     TrieCharString::from_vec_with_nul(b"abc\0".to_vec())
    ///         .expect("TrieCharString::from_vec_with_nul failed"),
    ///     TrieCharString::new(b"abc".to_vec()).expect("TrieCharString::new failed")
    /// );
    /// ```
    ///
    /// An incorrectly formatted [`Vec`] will produce an error.
    ///
    /// ```
    /// use datrie::trie_str::{TrieCharString, FromVecWithNulError};
    /// // Interior nul byte
    /// let _: FromVecWithNulError = TrieCharString::from_vec_with_nul(b"a\0bc".to_vec()).unwrap_err();
    /// // No nul byte
    /// let _: FromVecWithNulError = TrieCharString::from_vec_with_nul(b"abc".to_vec()).unwrap_err();
    /// ```
    pub fn from_vec_with_nul(v: Vec<u8>) -> Result<Self, FromVecWithNulError> {
        let nul_pos = memchr::memchr(0, &v);
        match nul_pos {
            Some(nul_pos) if nul_pos + 1 == v.len() => {
                // SAFETY: We know there is only one nul byte, at the end
                // of the vec.
                Ok(unsafe { Self::_from_vec_with_nul_unchecked(v) })
            }
            Some(nul_pos) => Err(FromVecWithNulError {
                error_kind: FromBytesWithNulErrorKind::InteriorNul(nul_pos),
                bytes: v,
            }),
            None => Err(FromVecWithNulError {
                error_kind: FromBytesWithNulErrorKind::NotNulTerminated,
                bytes: v,
            }),
        }
    }
}

impl fmt::Debug for TrieCharString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.to_bytes().escape_ascii())
    }
}
// Turns this `TrieCharString` into an empty string to prevent
// memory-unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl Drop for TrieCharString {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.inner.get_unchecked_mut(0) = 0;
        }
    }
}
impl Default for TrieCharString {
    /// Creates an empty `TrieCharString`.
    fn default() -> Self {
        let a: &TrieCharStr = Default::default();
        a.to_owned()
    }
}

// impl From<Vec<u8>> for TrieString {
//     fn from(value: Vec<u8>) -> Self {
//         Self { inner: value }
//     }
// }
// #[repr(tr)]
// pub struct TrieStr {
//     inner: [TrieChar],
// }
#[derive(PartialEq, Eq, Hash)]
// `fn from` in `impl From<&TrieCharStr> for Box<TrieCharStr>` current implementation relies
// on `TrieCharStr` being layout-compatible with `[u8]`.
// However, `TrieCharStr` layout is considered an implementation detail and must not be relied upon. We
// want `repr(transparent)` but we don't want it to show up in rustdoc, so we hide it under
// `cfg(doc)`. This is an ad-hoc implementation of attribute privacy.
#[repr(transparent)]
pub struct TrieCharStr {
    // FIXME: this should not be represented with a DST slice but rather with
    //        just a raw `c_char` along with some form of marker to make
    //        this an unsized type. Essentially `sizeof(&TrieCharStr)` should be the
    //        same as `sizeof(&c_char)` but `TrieCharStr` should be an unsized type.
    inner: [TrieChar],
}

/// An error indicating that a nul byte was not in the expected position.
///
/// The slice used to create a [`TrieCharStr`] must have one and only one nul byte,
/// positioned at the end.
///
/// This error is created by the [`TrieCharStr::from_bytes_with_nul`] method.
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use datrie::trie_str::{TrieCharStr, FromBytesWithNulError};
///
/// let _: FromBytesWithNulError = TrieCharStr::from_bytes_with_nul(b"f\0oo").unwrap_err();
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromBytesWithNulError {
    kind: FromBytesWithNulErrorKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum FromBytesWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

// FIXME: const stability attributes should not be required here, I think
impl FromBytesWithNulError {
    fn interior_nul(pos: usize) -> FromBytesWithNulError {
        FromBytesWithNulError {
            kind: FromBytesWithNulErrorKind::InteriorNul(pos),
        }
    }

    fn not_nul_terminated() -> FromBytesWithNulError {
        FromBytesWithNulError {
            kind: FromBytesWithNulErrorKind::NotNulTerminated,
        }
    }
}
impl fmt::Display for FromBytesWithNulError {
    #[allow(deprecated, deprecated_in_future)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())?;
        if let FromBytesWithNulErrorKind::InteriorNul(pos) = self.kind {
            write!(f, " at byte pos {pos}")?;
        }
        Ok(())
    }
}

impl Error for FromBytesWithNulError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self.kind {
            FromBytesWithNulErrorKind::InteriorNul(..) => {
                "data provided contains an interior nul byte"
            }
            FromBytesWithNulErrorKind::NotNulTerminated => "data provided is not nul terminated",
        }
    }
}
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
impl Default for &TrieCharStr {
    #[inline]
    fn default() -> Self {
        const SLICE: &[TrieChar] = &[0];
        // SAFETY: `SLICE` is indeed pointing to a valid nul-terminated string.
        unsafe { TrieCharStr::from_ptr(SLICE.as_ptr()) }
    }
}
impl TrieCharStr {
    #[inline] // inline is necessary for codegen to see strlen.
    #[must_use]
    pub unsafe fn from_ptr<'a>(ptr: *const TrieChar) -> &'a TrieCharStr {
        // SAFETY: The caller has provided a pointer that points to a valid C
        // string with a NUL terminator less than `isize::MAX` from `ptr`.
        let len = unsafe { libc::strlen(ptr as *const i8) };
        // dbg!(&len);

        // SAFETY: The caller has provided a valid pointer with length less than
        // `isize::MAX`, so `from_raw_parts` is safe. The content remains valid
        // and doesn't change for the lifetime of the returned `TrieCharStr`. This
        // means the call to `from_bytes_with_nul_unchecked` is correct.
        //
        // The cast from c_char to u8 is ok because a c_char is always one byte.
        unsafe { Self::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr.cast(), len + 1)) }
    }
    /// Creates a C string wrapper from a byte slice with any number of nuls.
    ///
    /// This method will create a `TrieCharStr` from any byte slice that contains at
    /// least one nul byte. Unlike with [`TrieCharStr::from_bytes_with_nul`], the caller
    /// does not need to know where the nul byte is located.
    ///
    /// If the first byte is a nul character, this method will return an
    /// empty `TrieCharStr`. If multiple nul characters are present, the `TrieCharStr` will
    /// end at the first one.
    ///
    /// If the slice only has a single nul byte at the end, this method is
    /// equivalent to [`TrieCharStr::from_bytes_with_nul`].
    ///
    /// # Examples
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let mut buffer = [0u8; 16];
    /// unsafe {
    ///     // Here we might call an unsafe C function that writes a string
    ///     // into the buffer.
    ///     let buf_ptr = buffer.as_mut_ptr();
    ///     buf_ptr.write_bytes(b'A', 8);
    /// }
    /// // Attempt to extract a C nul-terminated string from the buffer.
    /// let c_str = TrieCharStr::from_bytes_until_nul(&buffer[..]).unwrap();
    /// assert_eq!(c_str.to_bytes(), b"AAAAAAAA");
    /// ```
    ///
    pub fn from_bytes_until_nul(bytes: &[u8]) -> Result<&TrieCharStr, FromBytesUntilNulError> {
        let nul_pos = memchr::memchr(0, bytes);
        match nul_pos {
            Some(nul_pos) => {
                // FIXME(const-hack) replace with range index
                // SAFETY: nul_pos + 1 <= bytes.len()
                let subslice = unsafe { slice::from_raw_parts(bytes.as_ptr(), nul_pos + 1) };
                // SAFETY: We know there is a nul byte at nul_pos, so this slice
                // (ending at the nul byte) is a well-formed C string.
                Ok(unsafe { TrieCharStr::from_bytes_with_nul_unchecked(subslice) })
            }
            None => Err(FromBytesUntilNulError(())),
        }
    }

    /// Creates a C string wrapper from a byte slice with exactly one nul
    /// terminator.
    ///
    /// This function will cast the provided `bytes` to a `TrieCharStr`
    /// wrapper after ensuring that the byte slice is nul-terminated
    /// and does not contain any interior nul bytes.
    ///
    /// If the nul byte may not be at the end,
    /// [`TrieCharStr::from_bytes_until_nul`] can be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"hello\0");
    /// assert!(cstr.is_ok());
    /// ```
    ///
    /// Creating a `TrieCharStr` without a trailing nul terminator is an error:
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"hello");
    /// assert!(cstr.is_err());
    /// ```
    ///
    /// Creating a `TrieCharStr` with an interior nul byte is an error:
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"he\0llo\0");
    /// assert!(cstr.is_err());
    /// ```
    pub fn from_bytes_with_nul(bytes: &[u8]) -> Result<&Self, FromBytesWithNulError> {
        let nul_pos = memchr::memchr(0, bytes);
        match nul_pos {
            Some(nul_pos) if nul_pos + 1 == bytes.len() => {
                // SAFETY: We know there is only one nul byte, at the end
                // of the byte slice.
                Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
            }
            Some(nul_pos) => Err(FromBytesWithNulError::interior_nul(nul_pos)),
            None => Err(FromBytesWithNulError::not_nul_terminated()),
        }
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[TrieChar]) -> &TrieCharStr {
        // Chance at catching some UB at runtime with debug builds.
        // dbg!(bytes.len());
        // dbg!(&String::from_utf8_lossy(bytes));
        debug_assert!(!bytes.is_empty() && bytes[bytes.len() - 1] == 0);

        // SAFETY: Casting to TrieCharStr is safe because its internal representation
        // is a [u8] too (safe only inside std).
        // Dereferencing the obtained pointer is safe because it comes from a
        // reference. Making a reference is then safe because its lifetime
        // is bound by the lifetime of the given `bytes`.
        unsafe { &*(bytes as *const [TrieChar] as *const TrieCharStr) }
    }
    #[inline]
    #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
    pub fn to_bytes_with_nul(&self) -> &[TrieChar] {
        // SAFETY: Transmuting a slice of `c_char`s to a slice of `u8`s
        // is safe on all supported targets.
        unsafe { &*(addr_of!(self.inner) as *const [TrieChar]) }
    }
    /// Returns the inner pointer to this C string.
    ///
    /// The returned pointer will be valid for as long as `self` is, and points
    /// to a contiguous region of memory terminated with a 0 byte to represent
    /// the end of the string.
    ///
    /// The type of the returned pointer is
    /// [`*const c_char`][crate::ffi::c_char], and whether it's
    /// an alias for `*const i8` or `*const u8` is platform-specific.
    ///
    /// **WARNING**
    ///
    /// The returned pointer is read-only; writing to it (including passing it
    /// to C code that writes to it) causes undefined behavior.
    ///
    /// It is your responsibility to make sure that the underlying memory is not
    /// freed too early. For example, the following code will cause undefined
    /// behavior when `ptr` is used inside the `unsafe` block:
    ///
    /// ```no_run
    /// # #![allow(unused_must_use)]
    /// # #![cfg_attr(bootstrap, expect(temporary_cstring_as_ptr))]
    /// # #![cfg_attr(not(bootstrap), expect(dangling_pointers_from_temporaries))]
    /// use datrie::trie_str::TrieCharString;
    ///
    /// // Do not do this:
    /// let ptr = TrieCharString::new("Hello").expect("TrieCharString::new failed").as_ptr();
    /// unsafe {
    ///     // `ptr` is dangling
    ///     *ptr;
    /// }
    /// ```
    ///
    /// This happens because the pointer returned by `as_ptr` does not carry any
    /// lifetime information and the `TrieCharString` is deallocated immediately after
    /// the `TrieCharString::new("Hello").expect("TrieCharString::new failed").as_ptr()`
    /// expression is evaluated.
    /// To fix the problem, bind the `TrieCharString` to a local variable:
    ///
    /// ```no_run
    /// # #![allow(unused_must_use)]
    /// use datrie::trie_str::TrieCharString;
    ///
    /// let hello = TrieCharString::new("Hello").expect("TrieCharString::new failed");
    /// let ptr = hello.as_ptr();
    /// unsafe {
    ///     // `ptr` is valid because `hello` is in scope
    ///     *ptr;
    /// }
    /// ```
    ///
    /// This way, the lifetime of the `TrieCharString` in `hello` encompasses
    /// the lifetime of `ptr` and the `unsafe` block.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const TrieChar {
        self.inner.as_ptr()
    }
    /// Converts this C string to a byte slice.
    ///
    /// The returned slice will **not** contain the trailing nul terminator that this C
    /// string has.
    ///
    /// > **Note**: This method is currently implemented as a constant-time
    /// > cast, but it is planned to alter its definition in the future to
    /// > perform the length calculation whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"foo\0").expect("TrieCharStr::from_bytes_with_nul failed");
    /// assert_eq!(cstr.to_bytes(), b"foo");
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, \
                      without modifying the original"]
    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        // FIXME(const-hack) replace with range index
        // SAFETY: to_bytes_with_nul returns slice with length at least 1
        unsafe { slice::from_raw_parts(bytes.as_ptr(), bytes.len() - 1) }
    }
    /// Returns `true` if `self.to_bytes()` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    /// # use datrie::trie_str::FromBytesWithNulError;
    ///
    /// # fn main() { test().unwrap(); }
    /// # fn test() -> Result<(), FromBytesWithNulError> {
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"foo\0")?;
    /// assert!(!cstr.is_empty());
    ///
    /// let empty_cstr = TrieCharStr::from_bytes_with_nul(b"\0")?;
    /// assert!(empty_cstr.is_empty());
    /// assert!(c"".is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        // SAFETY: We know there is at least one byte; for empty strings it
        // is the NUL terminator.
        // FIXME(const-hack): use get_unchecked
        unsafe { *self.inner.as_ptr() == 0 }
    }

    /// Returns the length of `self`. Like C's `strlen`, this does not include the nul terminator.
    ///
    /// > **Note**: This method is currently implemented as a constant-time
    /// > cast, but it is planned to alter its definition in the future to
    /// > perform the length calculation whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::trie_str::TrieCharStr;
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"foo\0").unwrap();
    /// assert_eq!(cstr.count_bytes(), 3);
    ///
    /// let cstr = TrieCharStr::from_bytes_with_nul(b"\0").unwrap();
    /// assert_eq!(cstr.count_bytes(), 0);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias("len", "strlen"))]
    pub const fn count_bytes(&self) -> usize {
        self.inner.len() - 1
    }
}

impl fmt::Debug for TrieCharStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.to_bytes().escape_ascii())
    }
}
/// An error indicating that no nul byte was present.
///
/// A slice used to create a [`TrieCharStr`] must contain a nul byte somewhere
/// within the slice.
///
/// This error is created by the [`TrieCharStr::from_bytes_until_nul`] method.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromBytesUntilNulError(());

impl fmt::Display for FromBytesUntilNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data provided does not contain a nul")
    }
}
impl ops::Deref for TrieCharString {
    type Target = TrieCharStr;

    #[inline]
    fn deref(&self) -> &TrieCharStr {
        unsafe { TrieCharStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}
impl Borrow<TrieCharStr> for TrieCharString {
    #[inline]
    fn borrow(&self) -> &TrieCharStr {
        self
    }
}
impl ToOwned for TrieCharStr {
    type Owned = TrieCharString;

    fn to_owned(&self) -> TrieCharString {
        TrieCharString {
            inner: self.to_bytes_with_nul().into(),
        }
    }

    fn clone_into(&self, target: &mut TrieCharString) {
        let mut b = hack::into_vec(core::mem::take(&mut target.inner));
        self.to_bytes_with_nul().clone_into(&mut b);
        target.inner = b.into_boxed_slice();
    }
}

impl ops::Index<ops::RangeFrom<usize>> for TrieCharStr {
    type Output = TrieCharStr;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &TrieCharStr {
        let bytes = self.to_bytes_with_nul();
        // we need to manually check the starting index to account for the null
        // byte, since otherwise we could get an empty string that doesn't end
        // in a null.
        if index.start < bytes.len() {
            // SAFETY: Non-empty tail of a valid `TrieCharStr` is still a valid `TrieCharStr`.
            unsafe { TrieCharStr::from_bytes_with_nul_unchecked(&bytes[index.start..]) }
        } else {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                bytes.len(),
                index.start
            );
        }
    }
}

pub(crate) mod hack {

    // We shouldn't add inline attribute to this since this is used in
    // `vec!` macro mostly and causes perf regression. See #71204 for
    // discussion and perf results.
    #[allow(missing_docs)]
    pub fn into_vec<T>(b: Box<[T]>) -> Vec<T> {
        unsafe {
            let len = b.len();
            let b = Box::into_raw(b);
            Vec::from_raw_parts(b as *mut T, len, len)
        }
    }
}
