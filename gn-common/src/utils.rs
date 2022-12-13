pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    core::mem::discriminant(a) == core::mem::discriminant(b)
}

#[cfg(feature = "std")]
pub fn verification_msg<T: std::fmt::Display>(id: T) -> String {
    format!("This is my ({id}) registration request to Guild Network")
}

// NOTE: Panics if T is bigger than 64
// althought that shouldn't be a problem for now, since we're
// only using this for checking enum fields, which typically
// doesn't approach this number
pub fn detect_duplicates<T: Into<u64> + Copy>(iter: &[T]) -> bool {
    let mut bitvec = 0;
    for val in iter {
        let i = Into::<u64>::into(*val);
        if bitvec & (1 << i) == 0 {
            bitvec |= 1 << i;
        } else {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Copy)]
    enum Vars {
        Var1,
        Var2,
        Var3,
    }

    impl From<Vars> for u64 {
        fn from(value: Vars) -> Self {
            value as u64
        }
    }

    #[test]
    fn detect_dups() {
        let vec: Vec<u64> = vec![0, 0, 1, 2, 3];
        assert!(detect_duplicates(&vec));

        let vec: Vec<u64> = vec![1, 2, 3, 4, 5];
        assert!(!detect_duplicates(&vec));

        let vec: Vec<u64> = vec![0, 1, 2, 0, 3];
        assert!(detect_duplicates(&vec));

        let vec: Vec<u64> = vec![0, 1, 2, 0, 0];
        assert!(detect_duplicates(&vec));

        let vec: Vec<u64> = vec![];
        assert!(!detect_duplicates(&vec));
    }

    #[test]
    fn detect_dups_enum() {
        let vec: Vec<Vars> = vec![Vars::Var1, Vars::Var2, Vars::Var2, Vars::Var3];
        assert!(detect_duplicates(&vec));

        let vec: Vec<Vars> = vec![Vars::Var1, Vars::Var2, Vars::Var3];
        assert!(!detect_duplicates(&vec));
    }
}
