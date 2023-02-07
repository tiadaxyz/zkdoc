use crate::gadgets::poseidon::{PoseidonChip, PoseidonConfig};
use halo2_gadgets::{poseidon::primitives::P128Pow5T3, utilities::Var};
use halo2_proofs::{
    circuit::{floor_planner::V1, Layouter, Value},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

#[derive(Clone)]
pub struct FileHashPartialConfig {
    row_title_advice: Column<Advice>,
    row_content_advice: Column<Advice>,
    row_hash_advice: Column<Advice>,
    instance: Column<Instance>,
    poseidon_config: PoseidonConfig<3, 2, 2>,
}

pub struct FileHashPartialChip {
    config: FileHashPartialConfig,
}

impl FileHashPartialChip {
    pub fn construct(config: FileHashPartialConfig) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> FileHashPartialConfig {
        let row_content_advice = meta.advice_column();
        let row_title_advice = meta.advice_column();
        let row_hash_advice = meta.advice_column();
        let instance = meta.instance_column();

        meta.enable_equality(instance);

        let state = (0..3).map(|_| meta.advice_column()).collect::<Vec<_>>();
        for i in 0..3 {
            meta.enable_equality(state[i]);
        }

        let poseidon_config = PoseidonChip::<P128Pow5T3, 3, 2, 2>::configure(meta, state);

        FileHashPartialConfig {
            row_title_advice,
            row_content_advice,
            row_hash_advice,
            instance,
            poseidon_config,
        }
    }
}

pub struct FileHashPartialCircuit<const L: usize> {
    row_title: [Value<Fp>; L],
    row_content: [Value<Fp>; L],
}

impl<const L: usize> Circuit<Fp> for FileHashPartialCircuit<L> {
    type Config = FileHashPartialConfig;

    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self {
            row_title: (0..L)
                .map(|_i| Value::unknown())
                .collect::<Vec<Value<Fp>>>()
                .try_into()
                .unwrap(),
            row_content: (0..L)
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
        let poseidon_cs = PoseidonChip::<P128Pow5T3, 3, 2, 2>::construct(config.poseidon_config);

        let mut file_hashes = Vec::new();
        for i in 0..L {
            let message = [self.row_title[i], self.row_content[i]];
            let message_cells = poseidon_cs
                .load_private_inputs(layouter.namespace(|| "load private inputs"), message)?;
            let result =
                poseidon_cs.hash(layouter.namespace(|| "poseidon chip"), &message_cells)?;
            file_hashes.push(result);
        }

        let starting_poseidon_hash_message = [file_hashes[0].clone(), file_hashes[1].clone()];
        let mut accumulated_hash = poseidon_cs.hash(
            layouter.namespace(|| "poseidon chip"),
            &starting_poseidon_hash_message,
        )?;

        for i in 2..L {
            let message_cells = [accumulated_hash.clone(), file_hashes[i].clone()];
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
    use halo2_proofs::{circuit::Value, dev::MockProver, pasta::Fp};

    use super::FileHashPartialCircuit;

    #[test]
    fn test() {
        let k = 8;
        let row_title = [Fp::from(1), Fp::from(5)];
        let row_content = [Fp::from(3), Fp::from(4)];

        let circuit = FileHashPartialCircuit::<2> {
            row_title: row_title.map(|x| Value::known(x)),
            row_content: row_content.map(|x| Value::known(x)),
        };

        let mut row_hash = Vec::new();
        for (&title, &content) in row_title.iter().zip(row_content.iter()) {
            let message = [title, content];
            let output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

            row_hash.push(output);
        }

        let mut accumulator_hash = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
            .hash([row_hash[0], row_hash[1]]);

        for i in 2..row_content.len() {
            let message = [accumulator_hash, row_hash[i]];
            let output =
                poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);
            accumulator_hash = output;
        }

        let pub_instance = vec![accumulator_hash];
        let prover = MockProver::run(k, &circuit, vec![pub_instance]).unwrap();
        prover.assert_satisfied();
    }
}
