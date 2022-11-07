use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum TokenType<T, U> {
    Fungible { address: T },
    NonFungible { address: T, id: U },
}

#[derive(Deserialize, Serialize)]
pub enum Relation {
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

impl Relation {
    pub fn assert<T: PartialEq + PartialOrd>(&self, a: &T, b: &T) -> bool {
        match self {
            Relation::Equal => a == b,
            Relation::Greater => a > b,
            Relation::GreaterOrEqual => a >= b,
            Relation::Less => a < b,
            Relation::LessOrEqual => a <= b,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequiredBalance<T, U> {
    pub token_type: Option<TokenType<T, U>>,
    pub relation: Relation,
    pub amount: U,
}

#[cfg(test)]
mod test {
    use super::Relation;

    #[test]
    fn relations() {
        assert!(Relation::Equal.assert(&69, &69));
        assert!(!Relation::Equal.assert(&69, &420));
        assert!(!Relation::Equal.assert(&420, &23));

        assert!(Relation::Greater.assert(&420, &69));
        assert!(!Relation::Greater.assert(&69, &69));
        assert!(!Relation::Greater.assert(&23, &69));

        assert!(Relation::GreaterOrEqual.assert(&420, &23));
        assert!(Relation::GreaterOrEqual.assert(&23, &23));
        assert!(!Relation::GreaterOrEqual.assert(&14, &23));

        assert!(Relation::Less.assert(&1, &23));
        assert!(!Relation::Less.assert(&23, &23));
        assert!(!Relation::Less.assert(&420, &23));

        assert!(Relation::LessOrEqual.assert(&1, &2));
        assert!(Relation::LessOrEqual.assert(&23, &23));
        assert!(!Relation::LessOrEqual.assert(&420, &23));
    }
}
