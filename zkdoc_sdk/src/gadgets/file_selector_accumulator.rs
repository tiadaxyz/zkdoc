use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, Selector},
    poly::Rotation,
};

#[derive(Clone)]
pub struct FileSelectorAccumulatorConfig {
    col_a_advice: Column<Advice>,
    col_b_advice: Column<Advice>,
    selector: Selector,
}

pub struct FileSelectorAccumulatorChip<F> {
    config: FileSelectorAccumulatorConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FileSelectorAccumulatorChip<F> {
    pub fn construct(config: FileSelectorAccumulatorConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> FileSelectorAccumulatorConfig {
        let col_a_advice = meta.advice_column();
        let col_b_advice = meta.advice_column();
        let selector = meta.selector();

        meta.enable_equality(col_a_advice);
        meta.enable_equality(col_b_advice);

        meta.create_gate("accumulator gate", |region| {
            let col_a = region.query_advice(col_a_advice, Rotation::cur());
            let col_b = region.query_advice(col_b_advice, Rotation::cur());
            let col_a_res = region.query_advice(col_a_advice, Rotation::next());
            let s = region.query_selector(selector);

            Constraints::with_selector(s, [col_a + col_b - col_a_res])
        });
        FileSelectorAccumulatorConfig {
            col_a_advice,
            col_b_advice,
            selector,
        }
    }

    pub fn assign_first(
        &self,
        mut layouter: impl Layouter<F>,
        a: Value<F>,
        b: Value<F>,
        offset: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        let res = a + b;

        layouter.assign_region(
            || "selector accumulator assign first",
            |mut region| {
                self.config.selector.enable(&mut region, offset)?;

                let a_cell =
                    region.assign_advice(|| "a", self.config.col_a_advice, offset, || a)?;
                let b_cell =
                    region.assign_advice(|| "b", self.config.col_b_advice, offset, || b)?;
                let a_res_cell =
                    region.assign_advice(|| "a", self.config.col_a_advice, offset + 1, || res)?;

                Ok((a_cell, b_cell, a_res_cell))
            },
        )
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        prev_a_cell: &AssignedCell<F, F>,
        b: Value<F>,
        offset: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        let res = prev_a_cell.value().copied() + b;

        layouter.assign_region(
            || "selector accumulator assign",
            |mut region| {
                self.config.selector.enable(&mut region, offset)?;
                prev_a_cell.copy_advice(|| "a", &mut region, self.config.col_a_advice, offset)?;
                let b_cell =
                    region.assign_advice(|| "b", self.config.col_b_advice, offset, || b)?;
                let a_res_cell = region.assign_advice(
                    || "a res",
                    self.config.col_a_advice,
                    offset + 1,
                    || res,
                )?;

                Ok((b_cell, a_res_cell))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{
        arithmetic::FieldExt,
        circuit::{SimpleFloorPlanner, Value},
        dev::MockProver,
        pasta::Fp,
        plonk::Circuit,
    };

    use super::{FileSelectorAccumulatorChip, FileSelectorAccumulatorConfig};

    struct TestCircuit<F, const L: usize> {
        a: [Value<F>; L],
    }

    impl<F: FieldExt, const L: usize> Circuit<F> for TestCircuit<F, L> {
        type Config = FileSelectorAccumulatorConfig;

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                a: (0..L)
                    .map(|_i| Value::unknown())
                    .collect::<Vec<Value<F>>>()
                    .try_into()
                    .unwrap(),
            }
        }

        fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
            FileSelectorAccumulatorChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl halo2_proofs::circuit::Layouter<F>,
        ) -> Result<(), halo2_proofs::plonk::Error> {
            let cs = FileSelectorAccumulatorChip::construct(config);
            // first value
            let (_, _, mut prev_res) = cs.assign_first(
                layouter.namespace(|| "assign first"),
                self.a[0],
                self.a[1],
                0,
            )?;

            for i in 2..L {
                let (_, cur_res) = cs.assign(
                    layouter.namespace(|| "assign next"),
                    &prev_res,
                    self.a[i],
                    i - 1,
                )?;
                prev_res = cur_res;
            }

            Ok(())
        }
    }

    #[test]
    fn test() {
        let k = 8;
        let a = [Fp::from(0), Fp::from(1), Fp::from(2), Fp::from(3)];

        let circuit = TestCircuit {
            a: a.map(|i| Value::known(i)),
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
