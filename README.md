# Sketch for a more general Pattern API

### Presumably intended purpose of a more general Pattern API:

- searching through a 1-dimensional, slice-like data structures in a linear fashion
    - means we can use a model close to slices, or C++ iterators
- compatibility with iterator model
    - lazy progression possible
    - possibly double ended
- usable with mutable data
- should yield matches with minimum post-processing or bound checks necessary
- should be able to give position values relative to start of haystack
- Reusing implementations across the API
    - Needs abstract interface independend from slice type
    - Means being able to write a generic Split or Find function

### Possible extensions, but probably out of scope:

- searching through arbitrary n-dimensional data structures
    - would mean need for arbitrary search position type

### Result of iterativly making the design more general

Each has an implementation in a `::vN` submodule

- v1: str-only shared references with (usize, usize) search slice
  - Current Pattern API.
- v2: arbitrary slice type, shared references with (usize, usize) search slice.
- v3: arbitrary slice type, shared references with associated U search slice.
    - Allows replacing (usize, usize) with something more low level like
      (*const u8, *const u8).
    - (not impld): seperate Pattern API for mutable slices
        - Would require changes to haystack accessor,
          since there'd be mutable aliasing otherwise.
- v4: arbitrary `Ptr<slice>` type allowing mutability, with associated U search slice.
- v5: arbitrary `Ptr<slice>` type allowing mutability,
      with refactored associated types that work more similar to pointers /
      C++ iterators.
