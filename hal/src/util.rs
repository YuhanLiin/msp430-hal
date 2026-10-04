pub(crate) trait BitsExt {
    fn set(self, shift: u8) -> Self;
    fn check(self, shift: u8) -> Self;
    fn set_mask(self, mask: Self) -> Self;
}

impl BitsExt for u8 {
    #[inline(always)]
    fn set(self, shift: u8) -> Self { self | (1 << shift) }

    #[inline(always)]
    fn check(self, shift: u8) -> Self { self & (1 << shift) }

    #[inline(always)]
    fn set_mask(self, mask: Self) -> Self { self | mask }
}
