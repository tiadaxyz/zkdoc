use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
use halo2_proofs::{arithmetic::FieldExt, pasta::Fp};

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

fn convert_u32_to_u64_BE(u32_array: [u32; 2]) -> u64 {
    u32_array[0] as u64 * u64::pow(2, 32) + u32_array[1] as u64
}

fn convert_hash_u64_to_u32(hash_u64: [u64; 4]) -> [u32; 8] {
    let mut res = Vec::new();
    for num in hash_u64 {
        let res_u32 = convert_u64_to_u32_BE(num);
        res.push(res_u32[0]);
        res.push(res_u32[1]);
    }

    res.try_into().unwrap()
}

fn convert_u64_to_u32_BE(input: u64) -> [u32; 2] {
    let lower = input as u32;
    let upper = (input >> 32) as u32;

    [upper, lower]
}

// fn convert_hash_u32_to_u64(hash_u32: [u32; 8]) -> [u64; 4] {
//     let mut res = Vec::new();
//     for i in 0..4 {
//         let starting_index = i * 2;
//         let arr = [hash_u32[starting_index], hash_u32[starting_index + 1]];
//         let res_u64 = convert_u32_to_u64_BE(arr);
//         res.push(res_u64);
//     }

//     res.try_into().unwrap()
// }

// pub fn get_selected_row(row_title_js: JsValue, row_content_js: JsValue) -> JsValue {
//     let row_title_u32 = row_title_js.into_serde::<[u32; 8]>().unwrap();
//     let row_content_u32 = row_content_js.into_serde::<[u32; 8]>().unwrap();

//     let row_title_u64 = convert_hash_u32_to_u64(row_title_u32);
//     let row_content_u64 = convert_hash_u32_to_u64(row_content_u32);

//     let row_title = row_title_u64.map(|y| return Fp::from(y));
//     let row_content = row_content_u64.map(|y| return Fp::from(y));

//     let title_message_1 = [row_title[0], row_title[1]];
//     let title_message_2 = [row_title[2], row_title[3]];

//     let title_message_1_output =
//         poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_1);
//     let title_message_2_output =
//         poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_2);
//     let title_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
//         .hash([title_message_1_output, title_message_2_output]);

//     let content_message_1 = [row_content[0], row_content[1]];
//     let content_message_2 = [row_content[2], row_content[3]];

//     let content_message_1_output =
//         poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_1);
//     let content_message_2_output =
//         poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_2);
//     let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
//         .hash([content_message_1_output, content_message_2_output]);
//     let message = [title_hash, content_hash];
//     let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
// }

pub fn get_file_commitment_and_selected_row<const L: usize>(
    row_title: [[Fp; 4]; L],
    row_content: [[Fp; 4]; L],
    row_selector: [Fp; L],
) -> (Fp, Fp) {
    let mut row_hash = Vec::new();
    let mut row_accumulator = Fp::zero();
    for ((&title, &content), &row_selector) in row_title
        .iter()
        .zip(row_content.iter())
        .zip(row_selector.iter())
    {
        let title_message_1 = [title[0], title[1]];
        let title_message_2 = [title[2], title[3]];

        let title_message_1_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_1);
        let title_message_2_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_2);
        let title_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([title_message_1_output, title_message_2_output]);

        let content_message_1 = [content[0], content[1]];
        let content_message_2 = [content[2], content[3]];

        let content_message_1_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash(content_message_1);
        let content_message_2_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash(content_message_2);
        let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([content_message_1_output, content_message_2_output]);
        let message = [title_hash, content_hash];
        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

        row_hash.push(output);

        if row_selector == Fp::one() {
            row_accumulator += output;
        }
    }
    let mut file_commitment = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
        .hash([row_hash[0], row_hash[1]]);

    for i in 2..row_content.len() {
        let message = [file_commitment, row_hash[i]];
        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
        file_commitment = output;
    }

    // TODO, to test this result
    // let yy = file_commitment.get_lower_128();

    (file_commitment, row_accumulator)
}

#[cfg(test)]
mod tests {
    use crate::utils::poseidon::convert_hash_u64_to_u32;

    use super::convert_hash_u32_to_u64;
    // use bitvec::prelude::BitArray as A;

    use bitvec::{array::BitArray, order::Lsb0};
    use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
    use halo2_proofs::pasta::Fp;

    use ff::PrimeFieldBits;

    #[test]
    fn test() {
        let row_selector = [Fp::from(1), Fp::from(0)];

        let hash_1_u32: [u32; 8] = [
            1803989619, 4281662689, 2641068110, 4284104535, 1202562282, 2720996681, 3223212765,
            3079101259,
        ];

        let hash_1_u64 = convert_hash_u32_to_u64(hash_1_u32);
        println!("hash_1_u64: {:?}", hash_1_u64);
        let hash_1_fp = hash_1_u64.map(|x| return Fp::from(x));

        let row_title = [hash_1_fp, hash_1_fp];
        let row_content = [hash_1_fp, hash_1_fp];

        let mut row_hash = Vec::new();
        let mut row_accumulator = Fp::zero();
        for ((&title, &content), &row_selector) in row_title
            .iter()
            .zip(row_content.iter())
            .zip(row_selector.iter())
        {
            let title_message_1 = [title[0], title[1]];
            let title_message_2 = [title[2], title[3]];

            let title_message_1_output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                    .hash(title_message_1);
            let title_message_2_output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                    .hash(title_message_2);
            let title_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([title_message_1_output, title_message_2_output]);

            let content_message_1 = [content[0], content[1]];
            let content_message_2 = [content[2], content[3]];

            let content_message_1_output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                    .hash(content_message_1);
            let content_message_2_output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                    .hash(content_message_2);
            let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([content_message_1_output, content_message_2_output]);
            let message = [title_hash, content_hash];
            let output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

            row_hash.push(output);

            if row_selector == Fp::one() {
                row_accumulator += output;
            }
        }
        let mut accumulator_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([row_hash[0], row_hash[1]]);

        for i in 2..row_content.len() {
            let message = [accumulator_hash, row_hash[i]];
            let output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
            accumulator_hash = output;
        }

        println!("row_accumulator: {:?}", row_accumulator);
        println!(
            "row_accumulator_u64: {:?}",
            row_accumulator.to_le_bits().as_raw_slice()
        );
        println!(
            "row_accumulator_u32: {:?}",
            convert_hash_u64_to_u32(
                row_accumulator
                    .to_le_bits()
                    .as_raw_slice()
                    .try_into()
                    .unwrap()
            )
        );

        let xx = row_accumulator.to_le_bits();
        let yy = xx.as_raw_slice();
        let zz: [u64; 4] = yy.try_into().unwrap();
    }

    #[test]
    fn test2() {
        let row_title_u32 = [
            1803989619, 4281662689, 2641068110, 4284104535, 1202562282, 2720996681, 3223212765,
            3079101259,
        ];
        let row_content_u32 = row_title_u32.clone();

        let row_title_u64 = convert_hash_u32_to_u64(row_title_u32);
        let row_content_u64 = convert_hash_u32_to_u64(row_content_u32);

        let row_title = row_title_u64.map(|x| return Fp::from(x));
        let row_content = row_content_u64.map(|x| return Fp::from(x));

        let title_message_1 = [row_title[0], row_title[1]];
        let title_message_2 = [row_title[2], row_title[3]];

        let title_message_1_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_1);
        let title_message_2_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(title_message_2);
        let title_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([title_message_1_output, title_message_2_output]);

        let content_message_1 = [row_content[0], row_content[1]];
        let content_message_2 = [row_content[2], row_content[3]];

        let content_message_1_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash(content_message_1);
        let content_message_2_output =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash(content_message_2);
        let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([content_message_1_output, content_message_2_output]);
        let message = [title_hash, content_hash];
        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

        let output_le_bits: BitArray<[u64; 4]> = output.to_le_bits();
        let output_raw_slice = output_le_bits.as_raw_slice();

        println!("output_raw_slice: {:?}", output_raw_slice);

        let output_u32 = convert_hash_u64_to_u32(output_raw_slice.try_into().unwrap());
        let output_u32_2 = convert_hash_u64_to_u32(convert_to_u64_array(output_raw_slice));

        println!("output_u32: {:?}", output_u32);
        println!("output_u32_2: {:?}", output_u32_2);
    }

    fn convert_to_u64_array(input: &[u64]) -> [u64; 4] {
        let mut res = Vec::new();
        for i in 0..4 {
            res.push(input[i]);
        }

        res.try_into().unwrap()
    }
}
