use sha2::{Digest, Sha256};

pub fn get_sha256(input: &str) -> [u64;4] {
    let hashed = Sha256::digest(input);

    #[cfg(test)]
    {
        println!("hashed: {:x}", hashed);
    }

    let x: [u8; 32] = hashed.as_slice().try_into().expect("Wrong length");
    let pow_0 = u64::pow(256, 0);
    let pow_1 = u64::pow(256, 1);
    let pow_2 = u64::pow(256, 2);
    let pow_3 = u64::pow(256, 3);
    let pow_4 = u64::pow(256, 4);
    let pow_5 = u64::pow(256, 5);
    let pow_6 = u64::pow(256, 6);
    let pow_7 = u64::pow(256, 7);
    let mut res_u64 = Vec::new();
    for i in 0..4 {
        let starting_index = i * 8;
        let res = x[starting_index + 7] as u64 * pow_0
            + x[starting_index + 6] as u64 * pow_1
            + x[starting_index + 5] as u64 * pow_2
            + x[starting_index + 4] as u64 * pow_3
            + x[starting_index + 3] as u64 * pow_4
            + x[starting_index + 2] as u64 * pow_5
            + x[starting_index + 1] as u64 * pow_6
            + x[starting_index] as u64 * pow_7;
        res_u64.push(res);
    }

    #[cfg(test)]
    {
        println!("hashed_num: {:?}", res_u64);
    }


    let res_u64: [u64; 4] = res_u64.as_slice().try_into().unwrap();
    res_u64
}

#[cfg(test)]
mod tests {
    use super::get_sha256;

    #[test]
    fn test() {
        get_sha256("hello");
    }
}
