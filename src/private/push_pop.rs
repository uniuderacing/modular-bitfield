use crate::private::{
    checks::private::Sealed,
    PopBits,
    PushBits,
};

/// A bit buffer that allows to pop bits from it.
pub struct PopBuffer<T> {
    bytes: T,
}

impl<T> PopBuffer<T> {
    /// Creates a new pop buffer from the given bytes.
    #[inline]
    pub(super) fn from_bytes(bytes: T) -> Self {
        Self { bytes }
    }
}

impl Sealed for PopBuffer<u8> {}

impl PopBits for PopBuffer<u8> {
    #[inline]
    fn pop_bits(&mut self, amount: u32) -> u8 {
        let Self { bytes } = self;
        let orig_ones = bytes.count_ones();
        debug_assert!(1 <= amount && amount <= 8);
        let res = *bytes & ((0x01_u16.wrapping_shl(amount)).wrapping_sub(1) as u8);
        *bytes = bytes.checked_shr(amount).unwrap_or(0);
        debug_assert_eq!(res.count_ones() + bytes.count_ones(), orig_ones);
        res
    }
}

impl Sealed for PopBuffer<i8> {}

impl PopBits for PopBuffer<i8> {
    #[inline]
    fn pop_bits(&mut self, amount: u32) -> u8 {
        let Self { bytes } = self;
        let orig_ones = bytes.count_ones();
        debug_assert!(1 <= amount && amount <= 8);
        let res = (*bytes & ((0x01_i16.wrapping_shl(amount)).wrapping_sub(1) as i8)) as u8;
        *bytes = bytes.checked_shr(amount).unwrap_or(0);
        debug_assert_eq!(res.count_ones() + bytes.count_ones(), orig_ones);
        res
    }
}

macro_rules! impl_pop_bits {
    ( $($type:ty),+ ) => {
        $(
            impl Sealed for PopBuffer<$type> {}

            impl PopBits for PopBuffer<$type> {
                #[inline]
                fn pop_bits(&mut self, amount: u32) -> u8 {
                    let Self { bytes } = self;
                    let orig_ones = bytes.count_ones();
                    debug_assert!((1..=8).contains(&amount));
                    let bitmask = 0xFF >> (8 - amount);
                    let res = (*bytes & bitmask) as u8;
                    *bytes = bytes.checked_shr(amount).unwrap_or(0);

                    // Since Rust does arithmetic shifts, we need to mask the first `amount` bits to zero.
                    
                    // Let's build a block of `amount` ones.
                    let ones_block = (1 << amount) - 1;

                    // Shift those ones at the beginning of the block. Leave the rest as zeros.
                    let mask = ones_block << (core::mem::size_of::<$type>() * 8 - amount as usize);

                    // Mask the first `amount` bits to zero.
                    *bytes &= !mask;

                    debug_assert_eq!(res.count_ones() + bytes.count_ones(), orig_ones);
                    res
                }
            }
        )+
    };
}
impl_pop_bits!(u16, u32, u64, u128, i16, i32, i64, i128);

/// A bit buffer that allows to push bits onto it.
pub struct PushBuffer<T> {
    bytes: T,
}

impl<T> PushBuffer<T> {
    /// Returns the underlying bytes of the push buffer.
    #[inline]
    pub(super) fn into_bytes(self) -> T {
        self.bytes
    }
}

macro_rules! impl_push_bits {
    ( $($type:ty),+ ) => {
        $(
            impl Sealed for PushBuffer<$type> {}

            impl Default for PushBuffer<$type> {
                #[inline]
                fn default() -> Self {
                    Self { bytes: <$type as Default>::default() }
                }
            }

            impl PushBits for PushBuffer<$type> {
                #[inline]
                fn push_bits(&mut self, amount: u32, bits: u8) {
                    let Self { bytes } = self;
                    let orig_ones = bytes.count_ones();
                    debug_assert!(1 <= amount && amount <= 8);
                    let bitmask = 0xFF >> (8 - amount as u8);
                    *bytes = bytes.wrapping_shl(amount) | ((bits & bitmask) as $type);
                    debug_assert_eq!((bits & bitmask).count_ones() + orig_ones, bytes.count_ones());
                }
            }
        )+
    }
}
impl_push_bits!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
