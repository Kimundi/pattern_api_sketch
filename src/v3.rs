
pub trait Pattern<'a, H: ?Sized + SearchCursor>: Sized {
    type Searcher: Searcher<'a, H>;
    fn into_searcher(self, haystack: &'a H) -> Self::Searcher;
    fn is_prefix_of(self, haystack: &'a H) -> bool;
    fn is_suffix_of(self, haystack: &'a H) -> bool
        where Self::Searcher: ReverseSearcher<'a, H>;

    fn is_contained_in(self, haystack: &'a H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

pub trait SearchCursor {
    // Type specialized for both forward and backwards iteration through
    // a slice
    type SearchCursor;
}

pub unsafe trait Searcher<'a, H: ?Sized + SearchCursor> {
    fn haystack(&self) -> &'a H;

    fn next_match(&mut self) -> Option<H::SearchCursor>;
    fn next_reject(&mut self) -> Option<H::SearchCursor>;
}

pub unsafe trait ReverseSearcher<'a, H: ?Sized + SearchCursor>: Searcher<'a, H> {
    fn next_match_back(&mut self) -> Option<H::SearchCursor>;
    fn next_reject_back(&mut self) -> Option<H::SearchCursor>;
}

pub trait DoubleEndedSearcher<'a, H: ?Sized + SearchCursor>: ReverseSearcher<'a, H> {}


pub mod string {
    use super::*;
    impl SearchCursor for str {
        type SearchCursor = (*const u8, *const u8);
    }

    pub struct Ascii(u8);

    pub struct AsciiSearcher<'a> {
        haystack: &'a str,
        start: *const u8,
        end: *const u8,
        ascii: u8,
    }

    unsafe impl<'a> Searcher<'a, str> for AsciiSearcher<'a> {
        fn haystack(&self) -> &'a str { self.haystack }

        fn next_match(&mut self) -> Option<(*const u8, *const u8)> {
            while self.start != self.end {
                unsafe {
                    let p = self.start;
                    self.start = self.start.offset(1);

                    if *self.start == self.ascii {
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

                    if *self.start != self.ascii {
                        return Some((p, self.start));
                    }
                }
            }
            None
        }
    }

    impl<'a> Pattern<'a, str> for Ascii {
        type Searcher = AsciiSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
            AsciiSearcher {
                haystack: haystack,
                start: haystack.as_ptr(),
                end: unsafe {
                    haystack.as_ptr().offset(haystack.len() as isize)
                },
                ascii: self.0,
            }
        }

        fn is_prefix_of(self, haystack: &'a str) -> bool {
            haystack.as_bytes()
                .get(0)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }

        fn is_suffix_of(self, haystack: &'a str) -> bool
            where Self::Searcher: ReverseSearcher<'a, str> {
            haystack.as_bytes()
                .get(haystack.len() - 1)
                .map(|&b| b == self.0)
                .unwrap_or(false)
        }
    }

}

pub mod slice {
    //use super::*;
}

pub mod os_string {
    //use super::*;

}

pub mod generic {

}
