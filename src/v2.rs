
pub trait Pattern<'a, H: ?Sized>: Sized {
    type Searcher: Searcher<'a, H>;
    fn into_searcher(self, haystack: &'a H) -> Self::Searcher;
    fn is_prefix_of(self, haystack: &'a H) -> bool;
    fn is_suffix_of(self, haystack: &'a H) -> bool
        where Self::Searcher: ReverseSearcher<'a, H>;

    fn is_contained_in(self, haystack: &'a H) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }
}

pub unsafe trait Searcher<'a, H: ?Sized> {
    fn haystack(&self) -> &'a H;

    fn next_match(&mut self) -> Option<(usize, usize)>;
    fn next_reject(&mut self) -> Option<(usize, usize)>;
}

pub unsafe trait ReverseSearcher<'a, H: ?Sized>: Searcher<'a, H> {
    fn next_match_back(&mut self) -> Option<(usize, usize)>;
    fn next_reject_back(&mut self) -> Option<(usize, usize)>;
}

pub trait DoubleEndedSearcher<'a, H: ?Sized>: ReverseSearcher<'a, H> {}


pub mod string {
    use super::*;

    pub struct Ascii(u8);

    pub struct AsciiSearcher<'a> {
        haystack: &'a str,
        pos: usize,
        ascii: u8,
    }

    unsafe impl<'a> Searcher<'a, str> for AsciiSearcher<'a> {
        fn haystack(&self) -> &'a str { self.haystack }

        fn next_match(&mut self) -> Option<(usize, usize)> {
            loop {
                if let Some(&b) = self.haystack.as_bytes().get(self.pos) {
                    self.pos += 1;
                    if b == self.ascii {
                        return Some((self.pos - 1, self.pos));
                    } else {
                        continue;
                    }
                } else {
                    return None;
                }
            }
        }

        fn next_reject(&mut self) -> Option<(usize, usize)> {
            loop {
                if let Some(&b) = self.haystack.as_bytes().get(self.pos) {
                    self.pos += 1;
                    if b != self.ascii {
                        return Some((self.pos - 1, self.pos));
                    } else {
                        continue;
                    }
                } else {
                    return None;
                }
            }
        }
    }

    impl<'a> Pattern<'a, str> for Ascii {
        type Searcher = AsciiSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> Self::Searcher {
            AsciiSearcher {
                haystack: haystack,
                pos: 0,
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
