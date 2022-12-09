pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    core::mem::discriminant(a) == core::mem::discriminant(b)
}

#[cfg(feature = "std")]
pub fn verification_msg<T: std::fmt::Display>(id: T) -> String {
    format!("This is my ({id}) registration request to Guild Network")
}

// NOTE: Hic sunt dracones
pub fn check_for_duplicates<T: Into<u32> + Copy>(iter: &[T]) -> bool {
    let mut bitvec = 0;
    for val in iter {
        let i: u32 = (*val).into();
        if bitvec & (1 << i) == 0 {
            bitvec |= 1 << i;
        } else {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone)]
    enum Vars {
        Var1,
        Var2,
        Var3,
    }

    impl From<Vars> for u32 {
        fn from(value: Vars) -> Self {
            value as u32
        }
    }

    #[test]
    fn check_for_dups() {
        let vec: Vec<u32> = vec![1, 2, 3, 4, 5];
        assert!(check_for_duplicates(&vec));

        let vec: Vec<u32> = vec![0, 0, 1, 2, 3];
        assert!(!check_for_duplicates(&vec));
    }

    #[test]
    fn check_for_dups_enum() {
        let vec: Vec<Vars> = vec![Vars::Var1, Vars::Var2, Vars::Var3];
        assert!(check_for_duplicates(&vec));

        let vec: Vec<Vars> = vec![Vars::Var1, Vars::Var2, Vars::Var2, Vars::Var3];
        assert!(!check_for_duplicates(&vec));
    }
}
