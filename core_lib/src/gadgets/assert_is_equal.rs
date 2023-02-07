use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, Selector},
    poly::Rotation,
};

#[derive(Clone)]
pub struct AssertIsEqualConfig {
    left: Column<Advice>,
    right: Column<Advice>,
    selector: Selector,
}

pub struct AssertIsEqualChip<F> {
    config: AssertIsEqualConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> AssertIsEqualChip<F> {
    pub fn construct(config: AssertIsEqualConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> AssertIsEqualConfig {
        let left = meta.advice_column();
        let right = meta.advice_column();
        let selector = meta.selector();

        meta.create_gate("assert is equal gate", |region| {
            let s = region.query_selector(selector);
            let l = region.query_advice(left, Rotation::cur());
            let r = region.query_advice(right, Rotation::cur());

            Constraints::with_selector(s, [r - l])
        });

        AssertIsEqualConfig {
            left,
            right,
            selector,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        l: Value<F>,
        r: Value<F>,
        offset: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "assign assert is equal",
            |mut region| {
                self.config.selector.enable(&mut region, offset)?;

                let left_cell =
                    region.assign_advice(|| "left cell", self.config.left, offset, || l)?;

                let right_cell =
                    region.assign_advice(|| "right cell", self.config.right, offset, || r)?;
                Ok((left_cell, right_cell))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{
        arithmetic::FieldExt,
        circuit::{SimpleFloorPlanner, Value},
        pasta::Fp,
        plonk::Circuit, dev::MockProver,
    };

    use super::{AssertIsEqualChip, AssertIsEqualConfig};

    #[derive(Default)]
    struct TestCircuit<F> {
        left: Value<F>,
        right: Value<F>,
    }

    impl<F: FieldExt> Circuit<F> for TestCircuit<F> {
        type Config = AssertIsEqualConfig;

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
            AssertIsEqualChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl halo2_proofs::circuit::Layouter<F>,
        ) -> Result<(), halo2_proofs::plonk::Error> {
            let cs = AssertIsEqualChip::construct(config);

            cs.assign(layouter, self.left, self.right, 0)?;
            Ok(())
        }
    }

    #[test]
    fn test() {
        let k = 4;
        let left = Fp::from(500);
        let right = Fp::from(500);

        let circuit = TestCircuit {
            left: Value::known(left),
            right: Value::known(right)
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
