use std::marker::PhantomData;

// gadget to select row according to row selector
// also constraint the row selector to be boolean only
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, Expression, Selector},
    poly::Rotation,
};
#[derive(Clone)]
pub struct FileHashRowSelectorConfig {
    file_hash_advice: Column<Advice>,
    row_selector_advice: Column<Advice>,
    file_res_advice: Column<Advice>,
    selector: Selector,
}

pub struct FileHashRowSelectorChip<F> {
    config: FileHashRowSelectorConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FileHashRowSelectorChip<F> {
    pub fn construct(config: FileHashRowSelectorConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> FileHashRowSelectorConfig {
        let file_hash_advice = meta.advice_column();
        let row_selector_advice = meta.advice_column();
        let file_res_advice = meta.advice_column();
        let selector = meta.selector();

        meta.enable_equality(file_hash_advice);
        meta.enable_equality(file_res_advice);

        meta.create_gate("file hash row selector gate", |region| {
            let file_hash = region.query_advice(file_hash_advice, Rotation::cur());
            let row_selector = region.query_advice(row_selector_advice, Rotation::cur());
            let file_res = region.query_advice(file_res_advice, Rotation::cur());
            let s = region.query_selector(selector);

            Constraints::with_selector(
                s,
                [
                    file_hash * row_selector.clone() - file_res,
                    row_selector.clone() * (Expression::Constant(F::one()) - row_selector),
                ],
            )
        });

        FileHashRowSelectorConfig {
            file_hash_advice,
            row_selector_advice,
            file_res_advice,
            selector,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        file_hash: Value<F>,
        row_selector: Value<F>,
        offset: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "assign file selector",
            |mut region| {
                self.config.selector.enable(&mut region, offset)?;

                let file_hash_cell = region.assign_advice(
                    || "file hash",
                    self.config.file_hash_advice,
                    offset,
                    || file_hash,
                )?;

                let row_selector_cell = region.assign_advice(
                    || "row selector",
                    self.config.row_selector_advice,
                    offset,
                    || row_selector,
                )?;

                let file_res = file_hash * row_selector;

                let file_res_cell = region.assign_advice(
                    || "file res",
                    self.config.file_res_advice,
                    offset,
                    || file_res,
                )?;

                Ok((file_hash_cell, row_selector_cell, file_res_cell))
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

    use crate::gadgets::file_hash_row_selector::{
        FileHashRowSelectorChip, FileHashRowSelectorConfig,
    };

    struct TestCircuit<F, const L: usize> {
        file_hash: [Value<F>; L],
        row_selector: [Value<F>; L],
    }

    impl<F: FieldExt, const L: usize> Circuit<F> for TestCircuit<F, L> {
        type Config = FileHashRowSelectorConfig;

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                file_hash: (0..L)
                    .map(|_i| Value::unknown())
                    .collect::<Vec<Value<F>>>()
                    .try_into()
                    .unwrap(),
                row_selector: (0..L)
                    .map(|_i| Value::unknown())
                    .collect::<Vec<Value<F>>>()
                    .try_into()
                    .unwrap(),
            }
        }

        fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
            FileHashRowSelectorChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl halo2_proofs::circuit::Layouter<F>,
        ) -> Result<(), halo2_proofs::plonk::Error> {
            let cs = FileHashRowSelectorChip::construct(config);

            for i in 0..L {
                cs.assign(
                    layouter.namespace(|| "assign file hash row"),
                    self.file_hash[i],
                    self.row_selector[i],
                    i,
                )?;
            }

            Ok(())
        }
    }
    #[test]
    fn test() {
        let k = 4;
        let file_hashes = [Fp::from(5), Fp::from(7)];
        let file_selectors = [Fp::from(1), Fp::from(0)];

        let circuit = TestCircuit {
            file_hash: file_hashes.map(|hash| Value::known(hash)),
            row_selector: file_selectors.map(|selector| Value::known(selector)),
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fibo1() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("fib-1-layout.png", (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Fib 1 Layout", ("sans-serif", 60)).unwrap();

        let circuit = TestCircuit::<Fp, 2> {
            file_hash: [Value::unknown(), Value::unknown()],
            row_selector: [Value::unknown(), Value::unknown()],
        };
        halo2_proofs::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}
