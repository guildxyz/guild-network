use parity_scale_codec::alloc::string::String;

/// Panics if the input is more than 32 bytes long.
pub fn pad_to_32_bytes(input: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    input
        .as_bytes()
        .iter()
        .enumerate()
        .for_each(|(i, c)| output[i] = *c);
    output
}

pub fn unpad_from_32_bytes(input: &[u8; 32]) -> String {
    input
        .iter()
        .take_while(|&&x| x != 0)
        .map(|&x| char::from(x))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pad_works() {
        let input = "hello";
        let padded = pad_to_32_bytes(input);
        assert_eq!(&padded[..5], input.as_bytes());
        assert_eq!(&padded[5..], &[0u8; 27]);

        let input = "";
        let padded = pad_to_32_bytes(input);
        assert_eq!(padded, [0u8; 32]);

        let input = "this stuff here is 32 bytes long";
        let padded = pad_to_32_bytes(input);
        assert_eq!(&padded, input.as_bytes());
    }

    #[test]
    #[should_panic]
    fn pad_panics() {
        let input = "this stuff here is 33 bytes long!";
        pad_to_32_bytes(input);
    }

    #[test]
    fn unpad_works() {
        let input = "hello";
        let round_trip = unpad_from_32_bytes(&pad_to_32_bytes(input));
        assert_eq!(input, round_trip);

        let input = "";
        let round_trip = unpad_from_32_bytes(&pad_to_32_bytes(input));
        assert_eq!(input, round_trip);

        let input = "this is very nice";
        let round_trip = unpad_from_32_bytes(&pad_to_32_bytes(input));
        assert_eq!(input, round_trip);

        // TODO we should sanitize input to be valid utf8
        let input = "some é ö ü #$!";
        let round_trip = unpad_from_32_bytes(&pad_to_32_bytes(input));
        assert_ne!(input, round_trip);
    }
}
