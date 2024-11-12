use core::{fmt, ops, slice};
use std::error::Error;
use std::ptr::addr_of;

use crate::alpha_map::AlphaChar;

#[derive(PartialEq, Eq, Hash, Debug)]
// `fn from` in `impl From<&AlphaStr> for Box<AlphaStr>` current implementation relies
// on `AlphaStr` being layout-compatible with `[AlphaChar]`.
// However, `AlphaStr` layout is considered an implementation detail and must not be relied upon. We
// want `repr(transparent)` but we don't want it to show up in rustdoc, so we hide it under
// `cfg(doc)`. This is an ad-hoc implementation of attribute privacy.
#[repr(transparent)]
pub struct AlphaStr {
    // FIXME: this should not be represented with a DST slice but rather with
    //        just a raw `AlphaChar` along with some form of marker to make
    //        this an unsized type. Essentially `sizeof(&AlphaStr)` should be the
    //        same as `sizeof(&AlphaChar)` but `AlphaStr` should be an unsized type.
    inner: [AlphaChar],
}

/// An error indicating that a nul byte was not in the expected position.
///
/// The slice used to create a [`AlphaStr`] must have one and only one nul byte,
/// positioned at the end.
///
/// This error is created by the [`AlphaStr::from_slice_with_nul`] method.
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use datrie::alpha_str::{AlphaStr, FromSliceWithNulError};
///
/// let _: FromSliceWithNulError = AlphaStr::from_slice_with_nul(&['f' as u32, 0, 'o' as u32, 'o' as u32]).unwrap_err();
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromSliceWithNulError {
    kind: FromSliceWithNulErrorKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum FromSliceWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

// FIXME: const stability attributes should not be required here, I think
impl FromSliceWithNulError {
    const fn interior_nul(pos: usize) -> FromSliceWithNulError {
        FromSliceWithNulError {
            kind: FromSliceWithNulErrorKind::InteriorNul(pos),
        }
    }

    const fn not_nul_terminated() -> FromSliceWithNulError {
        FromSliceWithNulError {
            kind: FromSliceWithNulErrorKind::NotNulTerminated,
        }
    }
}

impl Error for FromSliceWithNulError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self.kind {
            FromSliceWithNulErrorKind::InteriorNul(..) => {
                "data provided contains an interior nul byte"
            }
            FromSliceWithNulErrorKind::NotNulTerminated => "data provided is not nul terminated",
        }
    }
}

/// An error indicating that no nul byte was present.
///
/// A slice used to create a [`AlphaStr`] must contain a nul byte somewhere
/// within the slice.
///
/// This error is created by the [`AlphaStr::from_slice_until_nul`] method.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromSliceUntilNulError(());

impl fmt::Display for FromSliceUntilNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data provided does not contain a nul")
    }
}

// impl fmt::Debug for AlphaStr {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "\"{:?}\"", self.to_slice().iter().map(|c| c as char))
//     }
// }

impl Default for &AlphaStr {
    #[inline]
    fn default() -> Self {
        const SLICE: &[AlphaChar] = &[0];
        // SAFETY: `SLICE` is indeed pointing to a valid nul-terminated string.
        unsafe { AlphaStr::from_ptr(SLICE.as_ptr()) }
    }
}

impl fmt::Display for FromSliceWithNulError {
    #[allow(deprecated, deprecated_in_future)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())?;
        if let FromSliceWithNulErrorKind::InteriorNul(pos) = self.kind {
            write!(f, " at byte pos {pos}")?;
        }
        Ok(())
    }
}

impl AlphaStr {
    /// Wraps a raw C string with a safe C string wrapper.
    ///
    /// This function will wrap the provided `ptr` with a `AlphaStr` wrapper, which
    /// allows inspection and interoperation of non-owned C strings. The total
    /// size of the terminated buffer must be smaller than [`isize::MAX`] **bytes**
    /// in memory (a restriction from [`slice::from_raw_parts`]).
    ///
    /// # Safety
    ///
    /// * The memory pointed to by `ptr` must contain a valid nul terminator at the
    ///   end of the string.
    ///
    /// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
    ///   This means in particular:
    ///
    ///     * The entire memory range of this `AlphaStr` must be contained within a single allocated object!
    ///     * `ptr` must be non-null even for a zero-length cstr.
    ///
    /// * The memory referenced by the returned `AlphaStr` must not be mutated for
    ///   the duration of lifetime `'a`.
    ///
    /// * The nul terminator must be within `isize::MAX` from `ptr`
    ///
    /// > **Note**: This operation is intended to be a 0-cost cast but it is
    /// > currently implemented with an up-front calculation of the length of
    /// > the string. This is not guaranteed to always be the case.
    ///
    /// # Caveat
    ///
    /// The lifetime for the returned slice is inferred from its usage. To prevent accidental misuse,
    /// it's suggested to tie the lifetime to whichever source lifetime is safe in the context,
    /// such as by providing a helper function taking the lifetime of a host value for the slice,
    /// or by explicit annotation.
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// use datrie::alpha_str::AlphaStr;
    /// use datrie::alpha_map::AlphaChar;
    ///
    /// // const HELLO_PTR: *const AlphaChar = {
    /// //    const BYTES: &[AlphaChar] = b"Hello, world!\0";
    /// //    BYTES.as_ptr().cast()
    /// // };
    /// // const HELLO: &AlphaStr = unsafe { AlphaStr::from_ptr(HELLO_PTR) };
    ///
    /// // assert_eq!(c"Hello, world!", HELLO);
    /// ```
    ///
    /// [valid]: core::ptr#safety
    #[inline] // inline is necessary for codegen to see strlen.
    #[must_use]
    pub unsafe fn from_ptr<'a>(ptr: *const AlphaChar) -> &'a AlphaStr {
        // SAFETY: The caller has provided a pointer that points to a valid C
        // string with a NUL terminator less than `isize::MAX` from `ptr`.
        let len = unsafe { alpha_char_strlen(ptr) };

        // SAFETY: The caller has provided a valid pointer with length less than
        // `isize::MAX`, so `from_raw_parts` is safe. The content remains valid
        // and doesn't change for the lifetime of the returned `AlphaStr`. This
        // means the call to `from_slice_with_nul_unchecked` is correct.
        //
        // The cast from AlphaChar to AlphaChar is ok because a AlphaChar is always one byte.
        unsafe { Self::from_slice_with_nul_unchecked(slice::from_raw_parts(ptr.cast(), len + 1)) }
    }

    /// Creates a C string wrapper from a byte slice with any number of nuls.
    ///
    /// This method will create a `AlphaStr` from any byte slice that contains at
    /// least one nul byte. Unlike with [`AlphaStr::from_slice_with_nul`], the caller
    /// does not need to know where the nul byte is located.
    ///
    /// If the first byte is a nul character, this method will return an
    /// empty `AlphaStr`. If multiple nul characters are present, the `AlphaStr` will
    /// end at the first one.
    ///
    /// If the slice only has a single nul byte at the end, this method is
    /// equivalent to [`AlphaStr::from_slice_with_nul`].
    ///
    /// # Examples
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let mut buffer = [0 as u32; 16];
    /// unsafe {
    ///     // Here we might call an unsafe C function that writes a string
    ///     // into the buffer.
    ///     let mut buf_ptr = buffer.as_mut_ptr();
    ///     for _ in 0..8 {
    ///         *buf_ptr = 'A' as u32;
    ///         buf_ptr = buf_ptr.offset(1);
    ///     }
    /// }
    /// // Attempt to extract a C nul-terminated string from the buffer.
    /// let c_str = AlphaStr::from_slice_until_nul(&buffer[..]).unwrap();
    /// assert_eq!(c_str.to_slice(), &['A' as u32, 'A' as u32, 'A' as u32, 'A' as u32, 'A' as u32, 'A' as u32, 'A' as u32, 'A' as u32]);
    /// ```
    ///
    pub fn from_slice_until_nul(bytes: &[AlphaChar]) -> Result<&AlphaStr, FromSliceUntilNulError> {
        // let mut buffer = [0 as u32; 16];
        // unsafe {
        //     let buf_ptr = buffer.as_mut_ptr();
        //     buf_ptr.write_s
        // }
        let nul_pos = alpha_char_memchr(0, bytes);
        match nul_pos {
            Some(nul_pos) => {
                // FIXME(const-hack) replace with range index
                // SAFETY: nul_pos + 1 <= bytes.len()
                let subslice = unsafe { slice::from_raw_parts(bytes.as_ptr(), nul_pos + 1) };
                // SAFETY: We know there is a nul byte at nul_pos, so this slice
                // (ending at the nul byte) is a well-formed C string.
                Ok(unsafe { AlphaStr::from_slice_with_nul_unchecked(subslice) })
            }
            None => Err(FromSliceUntilNulError(())),
        }
    }

    /// Creates a C string wrapper from a byte slice with exactly one nul
    /// terminator.
    ///
    /// This function will cast the provided `bytes` to a `AlphaStr`
    /// wrapper after ensuring that the byte slice is nul-terminated
    /// and does not contain any interior nul bytes.
    ///
    /// If the nul byte may not be at the end,
    /// [`AlphaStr::from_slice_until_nul`] can be used instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['h' as u32, 'e' as u32, 'l' as u32, 'l' as u32, 'o' as u32,0]);
    /// assert!(cstr.is_ok());
    /// ```
    ///
    /// Creating a `AlphaStr` without a trailing nul terminator is an error:
    ///
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['h' as u32, 'e' as u32, 'l' as u32, 'l' as u32, 'o' as u32]);
    /// assert!(cstr.is_err());
    /// ```
    ///
    /// Creating a `AlphaStr` with an interior nul byte is an error:
    ///
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['h' as u32, 'e' as u32, 0, 'l' as u32, 'l' as u32, 'o' as u32]);
    /// assert!(cstr.is_err());
    /// ```
    pub fn from_slice_with_nul(bytes: &[AlphaChar]) -> Result<&Self, FromSliceWithNulError> {
        let nul_pos = alpha_char_memchr(0, bytes);
        match nul_pos {
            Some(nul_pos) if nul_pos + 1 == bytes.len() => {
                // SAFETY: We know there is only one nul byte, at the end
                // of the byte slice.
                Ok(unsafe { Self::from_slice_with_nul_unchecked(bytes) })
            }
            Some(nul_pos) => Err(FromSliceWithNulError::interior_nul(nul_pos)),
            None => Err(FromSliceWithNulError::not_nul_terminated()),
        }
    }

    /// Unsafely creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `AlphaStr` wrapper without
    /// performing any sanity checks.
    ///
    /// # Safety
    /// The provided slice **must** be nul-terminated and not contain any interior
    /// nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::alpha_str::AlphaStr;
    ///
    /// unsafe {
    ///     let alpha_slice = &['f' as u32, 'o' as u32, 'o' as u32, 0];
    ///     let cstr = AlphaStr::from_slice_with_nul_unchecked(alpha_slice);
    ///     assert_eq!(cstr.to_slice_with_nul(), alpha_slice);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub unsafe fn from_slice_with_nul_unchecked(bytes: &[AlphaChar]) -> &AlphaStr {
        // Chance at catching some UB at runtime with debug builds.
        debug_assert!(!bytes.is_empty() && bytes[bytes.len() - 1] == 0);

        // SAFETY: Casting to AlphaStr is safe because its internal representation
        // is a [AlphaChar] too (safe only inside std).
        // Dereferencing the obtained pointer is safe because it comes from a
        // reference. Making a reference is then safe because its lifetime
        // is bound by the lifetime of the given `bytes`.
        unsafe { &*(bytes as *const [AlphaChar] as *const AlphaStr) }
    }

    /// Returns the inner pointer to this C string.
    ///
    /// The returned pointer will be valid for as long as `self` is, and points
    /// to a contiguous region of memory terminated with a 0 byte to represent
    /// the end of the string.
    ///
    /// The type of the returned pointer is
    /// [`*const AlphaChar`][crate::ffi::AlphaChar], and whether it's
    /// an alias for `*const i8` or `*const AlphaChar` is platform-specific.
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
    /// // use datrie::AlphaString;
    ///
    /// // Do not do this:
    /// // let ptr = AlphaString::new("Hello").expect("AlphaString::new failed").as_ptr();
    /// unsafe {
    ///     // `ptr` is dangling
    ///     //*ptr;
    /// }
    /// ```
    ///
    /// This happens because the pointer returned by `as_ptr` does not carry any
    /// lifetime information and the `AlphaString` is deallocated immediately after
    /// the `AlphaString::new("Hello").expect("AlphaString::new failed").as_ptr()`
    /// expression is evaluated.
    /// To fix the problem, bind the `AlphaString` to a local variable:
    ///
    /// ```no_run
    /// # #![allow(unused_must_use)]
    /// //use datrie::AlphaString;
    ///
    /// //let hello = AlphaStr::new("Hello").expect("AlphaString::new failed");
    /// //let ptr = hello.as_ptr();
    /// unsafe {
    ///     // `ptr` is valid because `hello` is in scope
    ///     //*ptr;
    /// }
    /// ```
    ///
    /// This way, the lifetime of the `AlphaString` in `hello` encompasses
    /// the lifetime of `ptr` and the `unsafe` block.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const AlphaChar {
        self.inner.as_ptr()
    }

    /// We could eventually expose this publicly, if we wanted.
    // #[inline]
    // #[must_use]
    // const fn as_non_null_ptr(&self) -> NonNull<AlphaChar> {
    //     // FIXME(const_trait_impl) replace with `NonNull::from`
    //     // SAFETY: a reference is never null
    //     unsafe { NonNull::new_unchecked(&self.inner as *const [AlphaChar] as *mut [AlphaChar]) }
    //         .as_non_null_ptr()
    // }

    /// Returns the length of `self`. Like C's `strlen`, this does not include the nul terminator.
    ///
    /// > **Note**: This method is currently implemented as a constant-time
    /// > cast, but it is planned to alter its definition in the future to
    /// > perform the length calculation whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['f' as u32, 'o' as u32, 'o' as u32, 0]).unwrap();
    /// assert_eq!(cstr.count_slice(), 3);
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&[0]).unwrap();
    /// assert_eq!(cstr.count_slice(), 0);
    /// ```
    #[inline]
    #[must_use]
    #[doc(alias("len", "strlen"))]
    pub const fn count_slice(&self) -> usize {
        self.inner.len() - 1
    }

    /// Returns `true` if `self.to_slice()` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::alpha_str::AlphaStr;
    /// # use datrie::alpha_str::FromSliceWithNulError;
    ///
    /// # fn main() { test().unwrap(); }
    /// # fn test() -> Result<(), FromSliceWithNulError> {
    /// let cstr = AlphaStr::from_slice_with_nul(&['f' as u32, 'o' as u32, 'o' as u32, 0])?;
    /// assert!(!cstr.is_empty());
    ///
    /// let empty_cstr = AlphaStr::from_slice_with_nul(&[0])?;
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
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['f' as u32, 'o' as u32, 'o' as u32, 0]).expect("AlphaStr::from_slice_with_nul failed");
    /// assert_eq!(cstr.to_slice(), &['f' as u32, 'o' as u32, 'o' as u32]);
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    pub const fn to_slice(&self) -> &[AlphaChar] {
        let bytes = self.to_slice_with_nul();
        // FIXME(const-hack) replace with range index
        // SAFETY: to_slice_with_nul returns slice with length at least 1
        unsafe { slice::from_raw_parts(bytes.as_ptr(), bytes.len() - 1) }
    }

    /// Converts this C string to a byte slice containing the trailing 0 byte.
    ///
    /// This function is the equivalent of [`AlphaStr::to_slice`] except that it
    /// will retain the trailing nul terminator instead of chopping it off.
    ///
    /// > **Note**: This method is currently implemented as a 0-cost cast, but
    /// > it is planned to alter its definition in the future to perform the
    /// > length calculation whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use datrie::AlphaStr;
    ///
    /// let cstr = AlphaStr::from_slice_with_nul(&['f' as u32, 'o' as u32, 'o' as u32, 0]).expect("AlphaStr::from_slice_with_nul failed");
    /// assert_eq!(cstr.to_slice_with_nul(), &['f' as u32, 'o' as u32, 'o' as u32,0]);
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    pub const fn to_slice_with_nul(&self) -> &[AlphaChar] {
        // SAFETY: Transmuting a slice of `AlphaChar`s to a slice of `AlphaChar`s
        // is safe on all supported targets.
        unsafe { &*((addr_of!(self.inner)) as *const [AlphaChar]) }
    }
}

impl ops::Index<ops::RangeFrom<usize>> for AlphaStr {
    type Output = AlphaStr;

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &AlphaStr {
        let bytes = self.to_slice_with_nul();
        // we need to manually check the starting index to account for the null
        // byte, since otherwise we could get an empty string that doesn't end
        // in a null.
        if index.start < bytes.len() {
            // SAFETY: Non-empty tail of a valid `AlphaStr` is still a valid `AlphaStr`.
            unsafe { AlphaStr::from_slice_with_nul_unchecked(&bytes[index.start..]) }
        } else {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                bytes.len(),
                index.start
            );
        }
    }
}

impl AsRef<AlphaStr> for AlphaStr {
    #[inline]
    fn as_ref(&self) -> &AlphaStr {
        self
    }
}

fn alpha_char_memchr(needle: AlphaChar, haystack: &[AlphaChar]) -> Option<usize> {
    for (idx, hay) in haystack.iter().enumerate() {
        if *hay == needle {
            return Some(idx);
        }
    }
    None
}
pub unsafe fn alpha_char_strlen(str: *const AlphaChar) -> usize {
    let mut p: *const AlphaChar = str;
    while *p != 0 {
        p = p.offset(1);
    }
    p.offset_from(str) as usize
}
