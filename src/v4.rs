
pub trait Pattern<H: SearchCursor>: Sized {
    type Searcher: Searcher<H>;
    fn into_searcher(self, haystack: H) -> Self::Searcher;
    fn is_prefix_of(self, haystack: H) -> bool;
    fn is_suffix_of(self, haystack: H) -> bool
        where Self::Searcher: ReverseSearcher<H>;

    fn is_contained_in(self, haystack: H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

pub trait SearchCursor {
    // Type specialized for both forward and backwards iteration through
    // a slice, and for better composability
    // with API consumers specific to the Self type.
    type Cursor: Copy;

    // State to recover the relative position from start of haystack
    // Is only guaranteed to be valid if combined with the
    // last yeilded Cursor from a Searcher
    type Start: Copy;

    unsafe fn offset_from_start(start: Self::Start, pos: Self::Cursor) -> usize;
    unsafe fn cursor_to_self(start: Self::Start, cursor: Self::Cursor) -> Self;
}

pub unsafe trait Searcher<H: SearchCursor> {
    fn haystack_start(&self) -> H::Start;

    fn next_match(&mut self) -> Option<H::Cursor>;
    fn next_reject(&mut self) -> Option<H::Cursor>;
}

pub unsafe trait ReverseSearcher<H: SearchCursor>: Searcher<H> {
    fn next_match_back(&mut self) -> Option<H::Cursor>;
    fn next_reject_back(&mut self) -> Option<H::Cursor>;
}

pub trait DoubleEndedSearcher<H: SearchCursor>: ReverseSearcher<H> {}


pub mod string {
    use super::*;
    impl<'a> SearchCursor for &'a str {
        type Cursor = (*const u8, *const u8);
        type Start = *const u8;
        unsafe fn offset_from_start(start: Self::Start,
                                    pos: Self::Cursor) -> usize
        {
            pos.0 as usize - start as usize
        }

        unsafe fn cursor_to_self(_: Self::Start,
                                 cursor: Self::Cursor) -> &'a str
        {
            let slice = ::std::slice::from_raw_parts(cursor.0,
                cursor.1 as usize - cursor.0 as usize);

            ::std::str::from_utf8_unchecked(slice)
        }
    }

    pub struct Ascii(pub u8);

    pub struct AsciiSearcher<'a> {
        front: *const u8,
        start: *const u8,
        end: *const u8,
        ascii: u8,
        _marker: ::std::marker::PhantomData<&'a str>
    }

    unsafe impl<'a> Searcher<&'a str> for AsciiSearcher<'a> {
        fn haystack_start(&self) -> *const u8 {
            self.front
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
            AsciiSearcher {
                front: haystack.as_ptr(),
                start: haystack.as_ptr(),
                end: unsafe {
                    haystack.as_ptr().offset(haystack.len() as isize)
                },
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
    impl<'a> SearchCursor for &'a mut [u8] {
        type Cursor = (*mut u8, *mut u8);

        // Store offset+1 from current position
        type Start = usize;

        unsafe fn offset_from_start(start: Self::Start, _: Self::Cursor) -> usize {
            start - 1
        }

        unsafe fn cursor_to_self(_: Self::Start,
                                 cursor: Self::Cursor) -> &'a mut [u8]
        {
            ::std::slice::from_raw_parts_mut(cursor.0,
                cursor.1 as usize - cursor.0 as usize)
        }
    }

    pub struct Ascii(pub u8);

    pub struct AsciiSearcher<'a> {
        front_offset: usize,
        start: *mut u8,
        end: *mut u8,
        ascii: u8,
        _marker: ::std::marker::PhantomData<&'a mut [u8]>
    }

    unsafe impl<'a> Searcher<&'a mut [u8]> for AsciiSearcher<'a> {
        fn haystack_start(&self) -> usize {
            self.front_offset
        }

        fn next_match(&mut self) -> Option<(*mut u8, *mut u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);
                    self.front_offset += 1;

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
                    self.front_offset += 1;

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
            AsciiSearcher {
                front_offset: 0,
                start: haystack.as_mut_ptr(),
                end: unsafe {
                    haystack.as_mut_ptr().offset(haystack.len() as isize)
                },
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
        where H: SearchCursor,
              P: Pattern<H>,
    {
        let mut searcher = pattern.into_searcher(haystack);
        let mut ret = vec![];

        while let Some(match_) = searcher.next_match() {
            let start = searcher.haystack_start();
            unsafe {
                let pos = H::offset_from_start(start, match_);
                let slice = H::cursor_to_self(start, match_);

                ret.push((pos, slice));
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

}
