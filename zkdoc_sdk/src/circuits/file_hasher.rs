use crate::gadgets::{
    file_hash_row_selector::{FileHashRowSelectorChip, FileHashRowSelectorConfig},
    file_selector_accumulator::{FileSelectorAccumulatorChip, FileSelectorAccumulatorConfig},
    poseidon::{PoseidonChip, PoseidonConfig},
};
use halo2_gadgets::poseidon::primitives::P128Pow5T3;
use halo2_proofs::{
    circuit::{floor_planner::V1, Value},
    pasta::Fp,
    plonk::{Circuit, Column, ConstraintSystem, Instance},
};

#[derive(Clone)]
pub struct FileHashPartialConfig {
    instance: Column<Instance>,
    poseidon_config: PoseidonConfig<3, 2, 2>,
    row_selector_config: FileHashRowSelectorConfig,
    row_selector_accumulator: FileSelectorAccumulatorConfig,
}

#[allow(unused)]
pub struct FileHashPartialChip {
    config: FileHashPartialConfig,
}

impl FileHashPartialChip {
    pub fn construct(config: FileHashPartialConfig) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> FileHashPartialConfig {
        let instance = meta.instance_column();

        meta.enable_equality(instance);

        let state = (0..3).map(|_| meta.advice_column()).collect::<Vec<_>>();
        for state in state.iter().take(3) {
            meta.enable_equality(*state);
        }

        let poseidon_config = PoseidonChip::<P128Pow5T3, 3, 2, 2>::configure(meta, state);
        let row_selector_config = FileHashRowSelectorChip::configure(meta);
        let row_selector_accumulator = FileSelectorAccumulatorChip::configure(meta);

        FileHashPartialConfig {
            instance,
            poseidon_config,
            row_selector_config,
            row_selector_accumulator,
        }
    }
}

#[derive(Clone)]
pub struct FileHashPartialCircuit<const L: usize> {
    pub row_title: [[Value<Fp>; 4]; L],
    pub row_content: [[Value<Fp>; 4]; L],
    pub row_selectors: [Value<Fp>; L],
}

impl<const L: usize> Circuit<Fp> for FileHashPartialCircuit<L> {
    type Config = FileHashPartialConfig;

    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self {
            row_title: (0..L)
                .map(|_i| [Value::unknown(); 4])
                .collect::<Vec<[Value<Fp>; 4]>>()
                .try_into()
                .unwrap(),
            row_content: (0..L)
                .map(|_i| [Value::unknown(); 4])
                .collect::<Vec<[Value<Fp>; 4]>>()
                .try_into()
                .unwrap(),
            row_selectors: (0..L)
                .map(|_i| Value::unknown())
                .collect::<Vec<Value<Fp>>>()
                .try_into()
                .unwrap(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        FileHashPartialChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fp>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        // get all row hashes
        let poseidon_cs = PoseidonChip::<P128Pow5T3, 3, 2, 2>::construct(config.poseidon_config);

        let mut file_hashes = Vec::new();
        for i in 0..L {
            let row_title_hash_1_message = [self.row_title[i][0], self.row_title[i][1]];
            let row_title_hash_2_message = [self.row_title[i][2], self.row_title[i][3]];

            // get row_title hash
            let row_title_hash_1_message_cell = poseidon_cs.load_private_inputs(
                layouter.namespace(|| "load private inputs"),
                row_title_hash_1_message,
            )?;
            let row_title_hash_1_message_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &row_title_hash_1_message_cell,
            )?;

            let row_title_hash_2_message_cell = poseidon_cs.load_private_inputs(
                layouter.namespace(|| "load private inputs"),
                row_title_hash_2_message,
            )?;
            let row_title_hash_2_message_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &row_title_hash_2_message_cell,
            )?;

            let row_title_hash_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &[
                    row_title_hash_1_message_result,
                    row_title_hash_2_message_result,
                ],
            )?;

            // get row_content hash
            let row_content_hash_1_message = [self.row_content[i][0], self.row_content[i][1]];
            let row_content_hash_2_message = [self.row_content[i][2], self.row_content[i][3]];

            // get row_title hash
            let row_content_hash_1_message_cell = poseidon_cs.load_private_inputs(
                layouter.namespace(|| "load private inputs"),
                row_content_hash_1_message,
            )?;
            let row_content_hash_1_message_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &row_content_hash_1_message_cell,
            )?;

            let row_content_hash_2_message_cell = poseidon_cs.load_private_inputs(
                layouter.namespace(|| "load private inputs"),
                row_content_hash_2_message,
            )?;
            let row_content_hash_2_message_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &row_content_hash_2_message_cell,
            )?;

            let row_content_hash_result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &[
                    row_content_hash_1_message_result,
                    row_content_hash_2_message_result,
                ],
            )?;

            // get file_hash
            let result = poseidon_cs.hash(
                layouter.namespace(|| "poseidon chip"),
                &[row_title_hash_result, row_content_hash_result],
            )?;
            file_hashes.push(result);
        }

        // multiply by row_selector
        let row_selector_cs = FileHashRowSelectorChip::<Fp>::construct(config.row_selector_config);

        let mut selected_rows = Vec::new();
        for (i, hash) in file_hashes.iter().enumerate() {
            let (file_hash_cell, _, file_res_cell) = row_selector_cs.assign(
                layouter.namespace(|| "row selectors"),
                hash.value().copied(),
                self.row_selectors[i],
                i,
            )?;

            layouter.assign_region(
                || "selector equality",
                |mut region| region.constrain_equal(file_hashes[i].cell(), file_hash_cell.cell()),
            )?;

            selected_rows.push(file_res_cell);
        }

        // accumulate selected row
        let row_selector_accumulator_cs =
            FileSelectorAccumulatorChip::<Fp>::construct(config.row_selector_accumulator);

        let (first_cell, second_cell, mut row_selector_accumulator) = row_selector_accumulator_cs
            .assign_first(
            layouter.namespace(|| "row selector accumulator first row"),
            selected_rows[0].value().copied(),
            selected_rows[1].value().copied(),
            0,
        )?;

        layouter.assign_region(
            || "accumulate assign first, first row",
            |mut region| {
                region.constrain_equal(first_cell.cell(), selected_rows[0].cell())?;
                region.constrain_equal(second_cell.cell(), selected_rows[1].cell())?;

                Ok(())
            },
        )?;

        for (i, selected) in selected_rows.iter().enumerate().take(L).skip(2) {
            let (b_cell, res_cell) = row_selector_accumulator_cs.assign(
                layouter.namespace(|| "row selector rest of the rows"),
                &row_selector_accumulator,
                selected.value().copied(),
                i - 1,
            )?;

            layouter.assign_region(
                || "accumulate assign rest of the rows",
                |mut region| {
                    region.constrain_equal(b_cell.cell(), selected_rows[i].cell())?;

                    Ok(())
                },
            )?;

            row_selector_accumulator = res_cell;
        }

        // expose row_selector_accumulator
        layouter.constrain_instance(row_selector_accumulator.cell(), config.instance, 1)?;

        // get final commitment
        let starting_poseidon_hash_message = [file_hashes[0].clone(), file_hashes[1].clone()];
        let mut accumulated_hash = poseidon_cs.hash(
            layouter.namespace(|| "poseidon chip"),
            &starting_poseidon_hash_message,
        )?;

        for hash in file_hashes.into_iter().take(L).skip(2) {
            let message_cells = [accumulated_hash.clone(), hash];
            accumulated_hash =
                poseidon_cs.hash(layouter.namespace(|| "poseidon chip"), &message_cells)?;
        }

        layouter.constrain_instance(accumulated_hash.cell(), config.instance, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
    use halo2_proofs::{
        circuit::Value,
        dev::MockProver,
        pasta::{EqAffine, Fp},
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier},
        poly::commitment::Params,
        transcript::{Blake2bRead, Blake2bWrite, Challenge255},
    };
    use rand_core::OsRng;

    use crate::utils::conversion::convert_hash_u32_to_u64;

    use super::FileHashPartialCircuit;
    use std::time::Instant;

    #[test]
    fn test() {
        let k = 20;
        let row_selector = [Fp::from(0), Fp::from(1)];

        let row_title = [
            [Fp::from(1), Fp::from(5), Fp::from(7), Fp::from(8)],
            [Fp::from(1), Fp::from(5), Fp::from(7), Fp::from(8)],
        ];
        let row_content = [
            [Fp::from(1), Fp::from(5), Fp::from(7), Fp::from(8)],
            [Fp::from(1), Fp::from(5), Fp::from(7), Fp::from(8)],
        ];

        let circuit = FileHashPartialCircuit::<2> {
            row_title: row_title.map(|x| x.map(|y| Value::known(y))),
            row_content: row_content.map(|x| x.map(|y| Value::known(y))),
            row_selectors: row_selector.map(|x| Value::known(x)),
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

        let pub_instance = vec![accumulator_hash, row_accumulator];
        let prover = MockProver::run(k, &circuit, vec![pub_instance]).unwrap();
        prover.assert_satisfied();
    }

    #[test]
    fn test_real_prover() {
        const ROW_NUMBER: usize = 14;
        let k = 13;

        let row_title_u32 = [
            1803989619, 4281662689, 2641068110, 4284104535, 1202562282, 2720996681, 3223212765,
            3079101259,
        ];
        let row_content_u32 = row_title_u32.clone();

        let row_title_u64 = convert_hash_u32_to_u64(row_title_u32);
        let row_content_u64 = convert_hash_u32_to_u64(row_content_u32);

        let row_title = row_title_u64.map(|x| return Fp::from(x));
        let row_content = row_content_u64.map(|x| return Fp::from(x));

        let row_title = [row_title; ROW_NUMBER];
        let row_content = [row_content; ROW_NUMBER];

        let row_selector_temp = [Fp::from(0); ROW_NUMBER - 1];
        let row_selector = [Fp::from(1)];

        let mut xx = row_selector.to_vec();
        let mut yy = row_selector_temp.to_vec();
        xx.append(&mut yy);
        let row_selector: [Fp; ROW_NUMBER] = xx.try_into().unwrap();

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

        for i in row_hash.into_iter().skip(2) {
            let message = [accumulator_hash, i];
            let output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
            accumulator_hash = output;
        }

        println!("accumulator_hash: {:?}", accumulator_hash);
        println!("row_accumulator: {:?}", row_accumulator);

        let public_input = vec![accumulator_hash, row_accumulator];
        let circuit = FileHashPartialCircuit::<ROW_NUMBER> {
            row_title: row_title.map(|x| x.map(|y| Value::known(y))),
            row_content: row_content.map(|x| x.map(|y| Value::known(y))),
            row_selectors: row_selector.map(|x| Value::known(x)),
        };

        let empty_circuit = FileHashPartialCircuit::<ROW_NUMBER> {
            row_title: [[Value::unknown(); 4]; ROW_NUMBER],
            row_content: [[Value::unknown(); 4]; ROW_NUMBER],
            row_selectors: [Value::unknown(); ROW_NUMBER],
        };
        let params: Params<EqAffine> = Params::new(k);

        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        // Create a proof
        let now = Instant::now();

        create_proof(
            &params,
            &pk,
            &[circuit.clone(), circuit.clone()],
            &[&[&public_input[..]], &[&public_input[..]]],
            OsRng,
            &mut transcript,
        )
        .expect("proof generation should not fail");
        let proof: Vec<u8> = transcript.finalize();
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?} for Row number: {:?}", elapsed, ROW_NUMBER);
        // println!("proof: {:?}", proof);

        let strategy = SingleVerifier::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        assert!(verify_proof(
            &params,
            pk.get_vk(),
            strategy,
            &[&[&public_input[..]], &[&public_input[..]]],
            &mut transcript,
        )
        .is_ok());
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fibo1() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("file-hasher-layout.png", (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("FileHasher Layout", ("sans-serif", 60))
            .unwrap();

        let circuit = FileHashPartialCircuit::<2> {
            row_title: [[Value::unknown(); 4]; 2],
            row_content: [[Value::unknown(); 4]; 2],
            row_selectors: [Value::unknown(); 2],
        };

        halo2_proofs::dev::CircuitLayout::default()
            .render(10, &circuit, &root)
            .unwrap();
    }
}
