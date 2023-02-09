pub fn convert_hash_u32_to_u64(hash_u32: [u32; 8]) -> [u64; 4] {
    let mut res = Vec::new();
    for i in 0..4 {
        let starting_index = i * 2;
        let arr = [hash_u32[starting_index], hash_u32[starting_index + 1]];
        let res_u64 = convert_u32_to_u64_BE(arr);
        res.push(res_u64);
    }

    res.try_into().unwrap()
}

pub fn convert_hash_u32_to_u64_LE(hash_u32: [u32; 8]) -> [u64; 4] {
    let mut res = Vec::new();
    for i in 0..4 {
        let starting_index = i * 2;
        let arr = [hash_u32[starting_index], hash_u32[starting_index + 1]];
        let res_u64 = convert_u32_to_u64_BE(arr);
        res.push(res_u64);
    }

    res.reverse();
    res.try_into().unwrap()
}

pub fn convert_u32_to_u64_BE(u32_array: [u32; 2]) -> u64 {
    u32_array[0] as u64 * u64::pow(2, 32) + u32_array[1] as u64
}

pub fn convert_hash_u64_to_u32(hash_u64: [u64; 4]) -> [u32; 8] {
    let mut res = Vec::new();
    for num in hash_u64 {
        let res_u32 = convert_u64_to_u32_BE(num);
        res.push(res_u32[0]);
        res.push(res_u32[1]);
    }

    res.try_into().unwrap()
}

pub fn convert_u64_to_u32_BE(input: u64) -> [u32; 2] {
    let lower = input as u32;
    let upper = (input >> 32) as u32;

    [upper, lower]
}

pub fn convert_to_u64_array(input: &[u64]) -> [u64; 4] {
    let mut res = Vec::new();
    for i in 0..4 {
        res.push(input[i]);
    }

    res.try_into().unwrap()
}