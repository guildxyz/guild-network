use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RequiredBalance<T, U, V> {
    pub token_type: Option<TokenType<T, U>>,
    pub relation: Relation<U>,
    pub chain: V,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TokenType<T, U> {
    Fungible { address: T },
    NonFungible { address: T, id: U },
    Special { address: T, metadata: Vec<u8> },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Relation<T> {
    EqualTo(T),
    GreaterThan(T),
    GreaterOrEqualTo(T),
    LessThan(T),
    LessOrEqualTo(T),
    Between(std::ops::Range<T>),
}

impl<T: PartialEq + PartialOrd> Relation<T> {
    pub fn assert(&self, x: &T) -> bool {
        match self {
            Relation::EqualTo(a) => x == a,
            Relation::GreaterThan(a) => x > a,
            Relation::GreaterOrEqualTo(a) => x >= a,
            Relation::LessThan(a) => x < a,
            Relation::LessOrEqualTo(a) => x <= a,
            Relation::Between(range) => range.contains(x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Relation;

    #[test]
    fn relations() {
        assert!(Relation::<u32>::EqualTo(0).assert(&0));
        assert!(!Relation::<u32>::EqualTo(10).assert(&2));
        assert!(!Relation::<u32>::EqualTo(420).assert(&421));

        assert!(!Relation::<u32>::GreaterThan(10).assert(&3));
        assert!(!Relation::<u32>::GreaterThan(10).assert(&10));
        assert!(Relation::<u32>::GreaterThan(10).assert(&20));

        assert!(Relation::<u32>::GreaterOrEqualTo(23).assert(&42));
        assert!(Relation::<u32>::GreaterOrEqualTo(23).assert(&23));
        assert!(!Relation::<u32>::GreaterOrEqualTo(23).assert(&14));

        assert!(Relation::<u32>::LessThan(23).assert(&1));
        assert!(!Relation::<u32>::LessThan(23).assert(&23));
        assert!(!Relation::<u32>::LessThan(23).assert(&42));

        assert!(Relation::<u32>::LessOrEqualTo(23).assert(&1));
        assert!(Relation::<u32>::LessOrEqualTo(23).assert(&23));
        assert!(!Relation::<u32>::LessOrEqualTo(23).assert(&42));

        assert!(!Relation::<u32>::Between(0..100).assert(&230));
        assert!(!Relation::<u32>::Between(50..100).assert(&15));
        assert!(!Relation::<u32>::Between(50..100).assert(&100)); // upper bound is not included
        assert!(Relation::<u32>::Between(50..100).assert(&77));
        assert!(Relation::<u32>::Between(50..100).assert(&50));
    }
}
