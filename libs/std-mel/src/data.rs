use melodium_core::{executive::*, *};
use melodium_macro::mel_type;

#[mel_type(
    traits (Binary Signed PartialEquality PartialOrder Order Euclid CheckedEuclid Pow CheckedNeg)
)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Structure {
    truc: String,
}

impl Binary for Structure {
    fn and(&self, other: &Self) -> Self {
        Structure {
            truc: "".to_string(),
        }
    }

    fn or(&self, other: &Self) -> Self {
        todo!()
    }

    fn xor(&self, other: &Self) -> Self {
        todo!()
    }

    fn not(&self) -> Self {
        todo!()
    }
}

impl Signed for Structure {
    fn abs(&self) -> Option<Self> {
        todo!()
    }

    fn signum(&self) -> Self {
        todo!()
    }

    fn is_positive(&self) -> bool {
        todo!()
    }

    fn is_negative(&self) -> bool {
        todo!()
    }
}

impl Euclid for Structure {
    fn euclid_div(&self, other: &Self) -> Self {
        todo!()
    }

    fn euclid_rem(&self, other: &Self) -> Self {
        todo!()
    }
}

impl CheckedEuclid for Structure {
    fn checked_euclid_div(&self, other: &Self) -> Option<Self> {
        todo!()
    }

    fn checked_euclid_rem(&self, other: &Self) -> Option<Self> {
        todo!()
    }
}

impl Pow for Structure {
    fn pow(&self, exp: &u32) -> Self {
        todo!()
    }
}

impl CheckedNeg for Structure {
    fn checked_neg(&self) -> Option<Self> {
        todo!()
    }
}
