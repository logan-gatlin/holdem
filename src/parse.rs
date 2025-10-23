pub trait Parse: Sized {
    fn parse(iter: &mut impl Iterator<Item = char>) -> Option<Self>;

    fn parse_n<const N: usize>(iter: &mut impl Iterator<Item = char>) -> Option<[Self; N]> {
        std::array::try_from_fn(|_| Self::parse(iter))
    }
}
