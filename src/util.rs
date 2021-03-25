//! Produces copies of N elements from the array
//! Yields a ~10% performance bump over returning pointers
pub struct ArrayChunksCopy<'a, T: 'a + Copy, const N: usize> {
    v: &'a [T],
    rem: &'a [T],
}

impl<'a, T: Copy, const N: usize> ArrayChunksCopy<'a, T, N> {
    #[inline]
    pub(super) fn new(slice: &'a [T]) -> Self {
        let rem = slice.len() % N;
        let fst_len = slice.len() - rem;
        // SAFETY: 0 <= fst_len <= slice.len() by construction above
        let (fst, snd) = slice.split_at(fst_len);
        Self { v: fst, rem: snd }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `chunk_size-1`
    /// elements.
    pub fn remainder(&self) -> &'a [T] {
        self.rem
    }
}

impl<T: Copy, const N: usize> Clone for ArrayChunksCopy<'_, T, N> {
    fn clone(&self) -> Self {
        ArrayChunksCopy {
            v: self.v,
            rem: self.rem,
        }
    }
}

impl<'a, T: Copy, const N: usize> Iterator for ArrayChunksCopy<'a, T, N> {
    type Item = [T; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.v.len() < N {
            None
        } else {
            let (fst, snd) = self.v.split_at(N);
            self.v = snd;
            Some(*unsafe { &*(fst as *const [T] as *const [T; N]) })
        }
    }
}

/// Stopgap until ArrayChunks is stabilized
pub struct ArrayChunks<'a, T: 'a, const N: usize> {
    v: &'a [T],
    rem: &'a [T],
}

impl<'a, T, const N: usize> ArrayChunks<'a, T, N> {
    #[inline]
    pub(super) fn new(slice: &'a [T]) -> Self {
        let rem = slice.len() % N;
        let fst_len = slice.len() - rem;
        // SAFETY: 0 <= fst_len <= slice.len() by construction above
        let (fst, snd) = slice.split_at(fst_len);
        Self { v: fst, rem: snd }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `chunk_size-1`
    /// elements.
    pub fn remainder(&self) -> &'a [T] {
        self.rem
    }
}

impl<T, const N: usize> Clone for ArrayChunks<'_, T, N> {
    fn clone(&self) -> Self {
        ArrayChunks {
            v: self.v,
            rem: self.rem,
        }
    }
}

impl<'a, T, const N: usize> Iterator for ArrayChunks<'a, T, N> {
    type Item = &'a [T; N];

    #[inline]
    fn next(&mut self) -> Option<&'a [T; N]> {
        if self.v.len() < N {
            None
        } else {
            let (fst, snd) = self.v.split_at(N);
            self.v = snd;
            Some(unsafe { &*(fst as *const [T] as *const [T; N]) })
        }
    }
}
