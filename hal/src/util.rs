pub(crate) trait BitsExt {
    fn set(self, shift: u8) -> Self;
    // Unused for now: remove this expect once it's used
    #[expect(dead_code)]
    fn clear(self, shift: u8) -> Self;
    fn check(self, shift: u8) -> Self;
    fn set_mask(self, mask: Self) -> Self;
    // Unused for now: remove this expect once it's used
    #[expect(dead_code)]
    fn clear_mask(self, mask: Self) -> Self;
}

impl BitsExt for u8 {
    #[inline(always)]
    fn set(self, shift: u8) -> Self { self | (1 << shift) }

    #[inline(always)]
    fn clear(self, shift: u8) -> Self { self & !(1 << shift) }

    #[inline(always)]
    fn check(self, shift: u8) -> Self { self & (1 << shift) }

    #[inline(always)]
    fn set_mask(self, mask: Self) -> Self { self | mask }

    #[inline(always)]
    fn clear_mask(self, mask: Self) -> Self { self & !mask }
}
