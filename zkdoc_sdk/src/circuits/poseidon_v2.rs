use crate::gadgets::poseidon_v2::{PoseidonChip as Chip, PoseidonConfig as Config};
use halo2_gadgets::poseidon::primitives::P128Pow5T3;
use halo2_proofs::{
    circuit::{floor_planner::V1, Value},
    pasta::Fp,
    plonk::{Circuit, ConstraintSystem},
};

#[derive(Clone)]
struct PoseidonConfig {
    poseidon_config: Config<3, 2, 2>,
}

#[allow(unused)]
struct PoseidonChip {
    config: PoseidonConfig,
}

impl PoseidonChip {
    #[allow(unused)]
    pub fn construct(config: PoseidonConfig) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<Fp>) -> PoseidonConfig {
        let poseidon_config = Chip::<P128Pow5T3, 3, 2, 2>::configure(meta);

        PoseidonConfig { poseidon_config }
    }
}

#[derive(Default)]
struct PoseidonCircuit {
    a: Value<Fp>,
    b: Value<Fp>,
}

impl Circuit<Fp> for PoseidonCircuit {
    type Config = PoseidonConfig;

    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        PoseidonChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fp>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let poseidon_cs = Chip::<P128Pow5T3, 3, 2, 2>::construct(config.poseidon_config);

        let message = [self.a, self.b];
        let message_cells = poseidon_cs
            .load_private_inputs(layouter.namespace(|| "load private inputs"), message)?;
        let result = poseidon_cs.hash(layouter.namespace(|| "poseidon chip"), &message_cells)?;
        poseidon_cs.expose_public(layouter.namespace(|| "expose result"), &result, 0)?;

        // layouter.constrain_instance(result.cell(), config.instance, 0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
    use halo2_proofs::{circuit::Value, dev::MockProver, pasta::Fp};

    use super::PoseidonCircuit;

    #[test]
    fn test() {
        let k = 8;
        let a = Fp::from(5);
        let b = Fp::from(7);

        let circuit = PoseidonCircuit {
            a: Value::known(a),
            b: Value::known(b),
        };

        let message = [a, b];

        let output = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

        let pub_instance = vec![output];

        let prover = MockProver::run(k, &circuit, vec![pub_instance]).unwrap();
        prover.assert_satisfied();
    }
}
