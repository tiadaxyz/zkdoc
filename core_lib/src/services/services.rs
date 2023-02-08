use crate::circuits::file_hasher::FileHashPartialCircuit;
use crate::utils::conversion::{
    convert_hash_u32_to_u64, convert_hash_u32_to_u64_LE, convert_hash_u64_to_u32,
    convert_to_u64_array, convert_u32_to_u64_BE, convert_u64_to_u32_BE,
};
use crate::utils::sha256::get_sha256;

use ff::PrimeFieldBits;
use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Value;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;

const ROW: usize = 10;

pub fn generate_row_hash(row_title_str: String, row_content_str: String) -> String {
    let row_title_u64 = get_sha256(row_title_str.as_str());
    let row_content_u64 = get_sha256(row_content_str.as_str());

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
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_1);
    let content_message_2_output =
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_2);
    let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
        .hash([content_message_1_output, content_message_2_output]);
    let message = [title_hash, content_hash];
    let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

    format!("{:?}", output)
}

pub fn get_file_commitment_and_selected_row(
    row_titles: [String; ROW],
    row_contents: [String; ROW],
    row_selectors: [u64; ROW],
) -> String {
    let row_title_u64 = row_titles.map(|x| return get_sha256(x.as_str()));
    let row_content_u64 = row_contents.map(|x| return get_sha256(x.as_str()));

    let row_title_fp = row_title_u64.map(|x| x.map(|y| return Fp::from(y)));
    let row_content_fp = row_content_u64.map(|x| x.map(|y| return Fp::from(y)));
    let row_selector_fp = row_selectors.map(|x| return Fp::from(x));

    let mut row_hash = Vec::new();
    let mut row_accumulator = Fp::zero();

    for ((&title, &content), &row_selector) in row_title_fp
        .iter()
        .zip(row_content_fp.iter())
        .zip(row_selector_fp.iter())
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

    for i in 2..row_content_fp.len() {
        let message = [file_commitment, row_hash[i]];
        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
        file_commitment = output;
    }

    format!("{:?}", file_commitment)
}

pub fn get_selected_row(row_title_str: [String; 8], row_content_str: [String; 8]) -> String {
    let row_title_u64 = get_sha256(row_title_str.as_str());
    let row_content_u64 = get_sha256(row_content_str.as_str());

    let row_title = row_title_u64.map(|y| return Fp::from(y));
    let row_content = row_content_u64.map(|y| return Fp::from(y));

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
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_1);
    let content_message_2_output =
        poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(content_message_2);
    let content_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
        .hash([content_message_1_output, content_message_2_output]);
    let message = [title_hash, content_hash];
    let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

    format!("{:?}", output)
}


pub fn generate_proof(
    row_title_str: [String; ROW],
    row_content_str: [String; ROW],
    row_selector_u64: [u64; ROW],
) -> Vec<u8> {
    let k = 12;
    let params: Params<EqAffine> = Params::new(k);

    let row_title_u64 = row_title_str.map(|x| return get_sha256(x.as_str()));
    let row_content_u64 = row_content_str.map(|x| return get_sha256(x.as_str()));

    let row_title = row_title_u64.map(|x| x.map(|y| return Fp::from(y)));
    let row_content = row_content_u64.map(|x| x.map(|y| return Fp::from(y)));
    let row_selector = row_selector_u64.map(|x| return Fp::from(x));

    let circuit = FileHashPartialCircuit::<ROW> {
        row_title: row_title.map(|x| x.map(|y| Value::known(y))),
        row_content: row_content.map(|x| x.map(|y| Value::known(y))),
        row_selectors: row_selector.map(|x| Value::known(x)),
    };

    let empty_circuit = FileHashPartialCircuit::<ROW> {
        row_title: [[Value::unknown(); 4]; ROW],
        row_content: [[Value::unknown(); 4]; ROW],
        row_selectors: [Value::unknown(); ROW],
    };

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
    let mut accumulator_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
        .hash([row_hash[0], row_hash[1]]);

    for i in 2..row_content.len() {
        let message = [accumulator_hash, row_hash[i]];
        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
        accumulator_hash = output;
    }

    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    let public_input = vec![accumulator_hash, row_accumulator];

    // Create a proof
    create_proof(
        &params,
        &pk,
        &[circuit.clone(), circuit.clone()],
        &[&[&public_input[..]], &[&public_input[..]]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation should not fail");

    transcript.finalize()
}

// pass in accumulator_hash and row_accumulator in the form of [u64;4]
pub fn verify_correct_selector(
    accumulator_hash: String,
    row_accumulator: String,
    proof: Vec<u8>,
) -> bool {
    // verify

    let k = 12;
    let params: Params<EqAffine> = Params::new(k);

    let accumulator_hash_u64_array = get_sha256(accumulator_hash.as_str()).into_iter().rev().collect();
    let row_accumulator_u64_array = get_sha256(row_accumulator.as_str()).into_iter().rev().collect();

    let accumulator_hash = Fp::from_raw(accumulator_hash_u64_array);
    let row_accumulator = Fp::from_raw(row_accumulator_u64_array);

    let empty_circuit = FileHashPartialCircuit::<ROW> {
        row_title: [[Value::unknown(); 4]; ROW],
        row_content: [[Value::unknown(); 4]; ROW],
        row_selectors: [Value::unknown(); ROW],
    };

    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    let public_input = vec![accumulator_hash, row_accumulator];

    verify_proof(
        &params,
        pk.get_vk(),
        strategy,
        &[&[&public_input[..]], &[&public_input[..]]],
        &mut transcript,
    )
    .is_ok()
}
