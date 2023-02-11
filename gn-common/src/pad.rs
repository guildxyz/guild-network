use parity_scale_codec::alloc::string::String;
use parity_scale_codec::alloc::vec::Vec;

/// Panics if the input is more than N bytes long.
pub fn pad_to_n_bytes<const N: usize, T: AsRef<[u8]>>(input: T) -> [u8; N] {
    let mut output = [0u8; N];
    input
        .as_ref()
        .iter()
        .enumerate()
        .for_each(|(i, c)| output[i] = *c);
    output
}

pub fn unpad_from_n_bytes<const N: usize>(input: &[u8; N]) -> String {
    input
        .iter()
        .take_while(|&&x| x != 0)
        .map(|&x| char::from(x))
        .collect()
}

/// Panics if the prefix bytes + 8 is more than 64 bytes
pub fn padded_id<T: AsRef<[u8]>>(prefix: T, id: u64) -> [u8; 64] {
    let mut tmp = Vec::from(prefix.as_ref());
    tmp.extend_from_slice(id.to_le_bytes().as_ref());
    pad_to_n_bytes(tmp)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pad_works() {
        let input = "hello";
        let padded = pad_to_n_bytes::<10, _>(input);
        assert_eq!(&padded[..5], input.as_bytes());
        assert_eq!(&padded[5..], &[0u8; 5]);

        let input = "";
        let padded = pad_to_n_bytes::<1, _>(input);
        assert_eq!(padded, [0u8]);

        let padded = pad_to_n_bytes::<0, _>(input);
        assert_eq!(&padded, input.as_bytes());

        let input = "this stuff here is 32 bytes long";
        let padded = pad_to_n_bytes::<32, _>(input);
        assert_eq!(&padded, input.as_bytes());
    }

    #[test]
    #[should_panic]
    fn pad_panics() {
        let input = "this stuff here is 33 bytes long!";
        pad_to_n_bytes::<32, _>(input);
    }

    #[test]
    fn unpad_works() {
        let input = "hello";
        let round_trip = unpad_from_n_bytes(&pad_to_n_bytes::<10, _>(input));
        assert_eq!(input, round_trip);

        let input = "";
        let round_trip = unpad_from_n_bytes(&pad_to_n_bytes::<2, _>(input));
        assert_eq!(input, round_trip);

        let input = "this is very nice";
        let round_trip = unpad_from_n_bytes(&pad_to_n_bytes::<32, _>(input));
        assert_eq!(input, round_trip);

        // TODO we should sanitize input to be valid utf8
        let input = "some é ö ü #$!";
        let round_trip = unpad_from_n_bytes(&pad_to_n_bytes::<64, _>(input));
        assert_ne!(input, round_trip);
    }
}
