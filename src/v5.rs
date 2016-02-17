
pub trait Pattern<H: SearchPtrs>: Sized {
    type Searcher: Searcher<H>;
    fn into_searcher(self, haystack: H) -> Self::Searcher;
    fn is_prefix_of(self, haystack: H) -> bool;
    fn is_suffix_of(self, haystack: H) -> bool
        where Self::Searcher: ReverseSearcher<H>;

    fn is_contained_in(self, haystack: H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

// Defined associated types and functions
// for dealing with positions in a slice-like type
// with pointer-like cursors
// Logically, Haystack <= Cursor <= Back
pub trait SearchPtrs {
    // For storing the bounds of the haystack.
    // Usually a combination of Memory address in form of a raw pointer or usize
    type Haystack: Copy;

    // Begin or End of a Match.
    // Two of these can be used to define a range of elements
    // as found by a Searcher.
    // Can be absolute, or relative to Haystack.
    // Usually a Memory address in form of a raw pointer or usize
    type Cursor: Copy;

    unsafe fn offset_from_start(hs: Self::Haystack, begin: Self::Cursor) -> usize;
    unsafe fn range_to_self(hs: Self::Haystack,
                            start: Self::Cursor,
                            end: Self::Cursor) -> Self;
    unsafe fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor;
    unsafe fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor;
}

pub unsafe trait Searcher<H: SearchPtrs> {
    fn haystack(&self) -> H::Haystack;

    fn next_match(&mut self) -> Option<(H::Cursor, H::Cursor)>;
    fn next_reject(&mut self) -> Option<(H::Cursor, H::Cursor)>;
}

pub unsafe trait ReverseSearcher<H: SearchPtrs>: Searcher<H> {
    fn next_match_back(&mut self) -> Option<(H::Cursor, H::Cursor)>;
    fn next_reject_back(&mut self) -> Option<(H::Cursor, H::Cursor)>;
}

pub trait DoubleEndedSearcher<H: SearchPtrs>: ReverseSearcher<H> {}


pub mod string {
    use super::*;
    impl<'a> SearchPtrs for &'a str {
        type Haystack = (*const u8, *const u8);
        type Cursor = *const u8;

        unsafe fn offset_from_start(haystack: Self::Haystack,
                                    begin: Self::Cursor) -> usize {
            begin as usize - haystack.0 as usize
        }

        unsafe fn range_to_self(_: Self::Haystack,
                                start: Self::Cursor,
                                end: Self::Cursor) -> Self {
            let slice = ::std::slice::from_raw_parts(start,
                end as usize - start as usize);

            ::std::str::from_utf8_unchecked(slice)
        }
        unsafe fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
            hs.0
        }
        unsafe fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
            hs.1
        }
    }

    pub struct Ascii(pub u8);

    pub struct AsciiSearcher<'a> {
        haystack: (*const u8, *const u8),
        start: *const u8,
        end: *const u8,
        ascii: u8,
        _marker: ::std::marker::PhantomData<&'a str>
    }

    unsafe impl<'a> Searcher<&'a str> for AsciiSearcher<'a> {
        fn haystack(&self) -> (*const u8, *const u8) {
            self.haystack
        }

        fn next_match(&mut self) -> Option<(*const u8, *const u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);

                    if *p == self.ascii {
                        return Some((p, self.start));
                    }
                }
            }
            None
        }

        fn next_reject(&mut self) -> Option<(*const u8, *const u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);

                    if *p != self.ascii {
                        return Some((p, self.start));
                    }
                }
            }
            None
        }
    }

    impl<'a> Pattern<&'a str> for Ascii {
        type Searcher = AsciiSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
            let begin = haystack.as_ptr();
            let end = unsafe {
                haystack.as_ptr().offset(haystack.len() as isize)
            };
            AsciiSearcher {
                haystack: (begin, end),
                start: begin,
                end: end,
                ascii: self.0,
                _marker: ::std::marker::PhantomData,
            }
        }

        fn is_prefix_of(self, haystack: &'a str) -> bool {
            haystack.as_bytes()
                .get(0)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }

        fn is_suffix_of(self, haystack: &'a str) -> bool
            where Self::Searcher: ReverseSearcher<&'a str> {
            haystack.as_bytes()
                .get(haystack.len() - 1)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }
    }

}

pub mod slice {
    use super::*;
    impl<'a> SearchPtrs for &'a mut [u8] {
        // Store address bounds as usize since aliasing interaction is unclear
        type Haystack = (*mut u8, *mut u8);
        type Cursor = *mut u8;

        unsafe fn offset_from_start(haystack: Self::Haystack,
                                    begin: Self::Cursor) -> usize {
            begin as usize - haystack.0 as usize
        }

        unsafe fn range_to_self(_: Self::Haystack,
                                start: Self::Cursor,
                                end: Self::Cursor) -> Self {
            ::std::slice::from_raw_parts_mut(start,
                end as usize - start as usize)
        }
        unsafe fn cursor_at_front(hs: Self::Haystack) -> Self::Cursor {
            hs.0 as *mut u8
        }
        unsafe fn cursor_at_back(hs: Self::Haystack) -> Self::Cursor {
            hs.1 as *mut u8
        }
    }

    pub struct Ascii(pub u8);

    pub struct AsciiSearcher<'a> {
        haystack: (*mut u8, *mut u8),
        start: *mut u8,
        end: *mut u8,
        ascii: u8,
        _marker: ::std::marker::PhantomData<&'a mut [u8]>
    }

    unsafe impl<'a> Searcher<&'a mut [u8]> for AsciiSearcher<'a> {
        fn haystack(&self) -> (*mut u8, *mut u8) {
            self.haystack
        }

        fn next_match(&mut self) -> Option<(*mut u8, *mut u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);

                    if *p == self.ascii {
                        return Some((p, self.start));
                    }
                }
            }
            None
        }

        fn next_reject(&mut self) -> Option<(*mut u8, *mut u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);

                    if *p != self.ascii {
                        return Some((p, self.start));
                    }
                }
            }
            None
        }
    }

    impl<'a> Pattern<&'a mut [u8]> for Ascii {
        type Searcher = AsciiSearcher<'a>;

        fn into_searcher(self, haystack: &'a mut [u8]) -> Self::Searcher {
            let begin = haystack.as_mut_ptr();
            let end = unsafe {
                haystack.as_mut_ptr().offset(haystack.len() as isize)
            };

            AsciiSearcher {
                haystack: (begin, end),
                start: begin,
                end: end,
                ascii: self.0,
                _marker: ::std::marker::PhantomData,
            }
        }

        fn is_prefix_of(self, haystack: &'a mut [u8]) -> bool {
            haystack
                .get(0)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }

        fn is_suffix_of(self, haystack: &'a mut [u8]) -> bool
            where Self::Searcher: ReverseSearcher<&'a mut [u8]> {
            haystack
                .get(haystack.len() - 1)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }
    }
}

pub mod os_string {
    //use super::*;

}

pub mod api_consumer {
    use super::*;

    pub fn match_indices<H, P>(haystack: H, pattern: P) -> Vec<(usize, H)>
        where H: SearchPtrs,
              P: Pattern<H>,
    {
        let mut searcher = pattern.into_searcher(haystack);
        let mut ret = vec![];

        while let Some((begin, end)) = searcher.next_match() {
            let haystack = searcher.haystack();
            unsafe {
                let offset = H::offset_from_start(haystack, begin);
                let slice = H::range_to_self(haystack, begin, end);

                ret.push((offset, slice));
            }
        }

        ret
    }

    #[test]
    fn test_match_indices() {
        assert_eq!(match_indices("banana", string::Ascii(b'a')),
                   vec![(1, "a"), (3, "a"), (5, "a")]);

        let mut slice = &mut {*b"banana"}[..];

        {
            let match_indices = match_indices(&mut*slice, slice::Ascii(b'a'));

            assert_eq!(match_indices.iter().map(|x| x.0).collect::<Vec<_>>(),
                       vec![1, 3, 5]);

            for m in match_indices {
                m.1[0] = b'i';
            }
        }

        assert_eq!(slice, b"binini");
    }

    pub fn split<H, P>(haystack: H, pattern: P) -> Vec<H>
        where H: SearchPtrs,
              P: Pattern<H>,
    {
        let mut searcher = pattern.into_searcher(haystack);
        let mut ret = vec![];

        let haystack = searcher.haystack();

        let mut last_end = Some(unsafe {
            H::cursor_at_front(haystack)
        });

        while let Some((begin, end)) = searcher.next_match() {
            if let Some(last_end) = last_end {
                unsafe {
                    let slice = H::range_to_self(haystack, last_end, begin);
                    ret.push(slice);
                }
            }
            last_end = Some(end);
        }

        if let Some(last_end) = last_end {
            unsafe {
                let end = H::cursor_at_back(haystack);
                let slice = H::range_to_self(haystack, last_end, end);
                ret.push(slice);
            }
        }

        ret
    }

    #[test]
    fn test_split() {
        assert_eq!(split("hangman", string::Ascii(b'a')),
                   vec!["h", "ngm", "n"]);

        let mut slice = &mut {*b"hangman"}[..];

        {
            let split = split(&mut*slice, slice::Ascii(b'a'));

            for m in split {
                for byte in m {
                    *byte = b'-';
                }
            }
        }

        assert_eq!(slice, b"-a---a-");
    }

}
