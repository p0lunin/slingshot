#![allow(non_snake_case)]

use bulletproofs::circuit_proof::assignment::Assignment;
use bulletproofs::circuit_proof::r1cs::{ConstraintSystem, LinearCombination, Variable};
use curve25519_dalek::scalar::Scalar;
use util::Value;

pub struct Shuffle {}

impl Shuffle {
    fn fill_cs(
        cs: &mut ConstraintSystem,
        in_0: (Variable, Assignment),
        in_1: (Variable, Assignment),
        out_0: (Variable, Assignment),
        out_1: (Variable, Assignment),
    ) {
        let one = Scalar::one();
        let zer = Scalar::zero();
        let w = cs.challenge_scalar(b"shuffle challenge");

        // create variables for multiplication 1
        let (mul1_left, mul1_right, mul1_out) =
            cs.assign_multiplier(in_0.1 - w, in_1.1 - w, (in_0.1 - w) * (in_1.1 - w));
        // mul1_left = in_0 - w
        cs.add_constraint(LinearCombination::new(
            vec![(mul1_left, -one), (in_0.0, one)],
            -w,
        ));
        // mul1_right = in_1 - w
        cs.add_constraint(LinearCombination::new(
            vec![(mul1_right, -one), (in_1.0, one)],
            -w,
        ));

        // create variables for multiplication 2
        let (mul2_left, mul2_right, mul2_out) =
            cs.assign_multiplier(out_0.1 - w, out_1.1 - w, (out_0.1 - w) * (out_1.1 - w));
        // mul2_left = out_0 - w
        cs.add_constraint(LinearCombination::new(
            vec![(mul2_left, -one), (out_0.0, one)],
            -w,
        ));
        // mul2_right = out_1 - w
        cs.add_constraint(LinearCombination::new(
            vec![(mul2_right, -one), (out_1.0, one)],
            -w,
        ));
        // mul1_out = mul2_out
        cs.add_constraint(LinearCombination::new(
            vec![(mul1_out, one), (mul2_out, -one)],
            zer,
        ));
    }
}

pub struct Merge {}

impl Merge {
    fn fill_cs(cs: &mut ConstraintSystem, A: Value, B: Value, C: Value, D: Value) {
        let one = Scalar::one();
        let zer = Scalar::zero();
        let w = cs.challenge_scalar(b"merge challenge");

        // create variables for multiplication
        let (mul_left, mul_right, mul_out) = cs.assign_multiplier(
            // left gate to multiplier
            (A.q.1 - C.q.1)
                + (A.a.1 - C.a.1) * w
                + (A.t.1 - C.t.1) * w * w
                + (B.q.1 - D.q.1) * w * w * w
                + (B.a.1 - D.a.1) * w * w * w * w
                + (B.t.1 - D.t.1) * w * w * w * w * w,
            // right gate to multiplier
            C.q.1
                + (A.a.1 - B.a.1) * w
                + (A.t.1 - B.t.1) * w * w
                + (D.q.1 - A.q.1 + B.q.1) * w * w * w
                + (D.a.1 - A.a.1) * w * w * w * w
                + (D.t.1 - A.t.1) * w * w * w * w * w,
            // out gate to multiplier
            Assignment::zero(),
        );
        // mul_left  = (A.q - C.q) +
        //             (A.a - C.a) * w +
        //             (A.t - C.t) * w^2 +
        //             (B.q - D.q) * w^3 +
        //             (B.a - D.a) * w^4 +
        //             (B.t - D.t) * w^5
        cs.add_constraint(LinearCombination::new(
            vec![
                (mul_left, -one),
                (A.q.0, one),
                (C.q.0, -one),
                (A.a.0, w),
                (C.a.0, -w),
                (A.t.0, w * w),
                (C.t.0, -w * w),
                (B.q.0, w * w * w),
                (D.q.0, -w * w * w),
                (B.a.0, w * w * w * w),
                (D.a.0, -w * w * w * w),
                (B.t.0, w * w * w * w * w),
                (D.t.0, -w * w * w * w * w),
            ],
            zer,
        ));
        // mul_right = (C.q - 0) +
        //             (A.a - B.a) * w +
        //             (A.t - B.t) * w^2 +
        //             (D.q - A.q + B.q) * w^3 +
        //             (D.a - A.a) * w^4
        //             (D.t - A.t) * w^5
        cs.add_constraint(LinearCombination::new(
            vec![
                (mul_right, -one),
                (C.q.0, one),
                (A.a.0, w),
                (B.a.0, -w),
                (A.t.0, w * w),
                (B.t.0, -w * w),
                (D.q.0, w * w * w),
                (A.q.0, -w * w * w),
                (B.q.0, w * w * w),
                (D.a.0, w * w * w * w),
                (A.a.0, -w * w * w * w),
                (D.t.0, w * w * w * w * w),
                (A.t.0, -w * w * w * w * w),
            ],
            zer,
        ));
        // mul_out   = 0
        cs.add_constraint(LinearCombination::new(vec![(mul_out, -one)], zer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bulletproofs::R1CSError;
    use bulletproofs::{Generators, PedersenGenerators, Transcript};
    use rand::rngs::OsRng;

    #[test]
    fn shuffle_gadget() {
        assert!(shuffle_helper(3, 6, 3, 6).is_ok());
        assert!(shuffle_helper(3, 6, 6, 3).is_ok());
        assert!(shuffle_helper(6, 6, 6, 6).is_ok());
        assert!(shuffle_helper(3, 3, 6, 3).is_err());
    }

    fn shuffle_helper(in_0: u64, in_1: u64, out_0: u64, out_1: u64) -> Result<(), R1CSError> {
        let mut rng = OsRng::new().unwrap();

        // Prover makes a `ConstraintSystem` instance representing a shuffle gadget
        // v and v_blinding emptpy because we are only testing low-level variable constraints
        let v = vec![];
        let v_blinding = vec![];
        let mut prover_transcript = Transcript::new(b"ShuffleTest");
        let (mut prover_cs, _committed_variables, commitments) = ConstraintSystem::prover_new(
            &mut prover_transcript,
            v,
            v_blinding.clone(),
            PedersenGenerators::default(),
        );
        // Prover allocates variables and adds constraints to the constraint system
        shuffle_cs(
            &mut prover_cs,
            Assignment::from(in_0),
            Assignment::from(in_1),
            Assignment::from(out_0),
            Assignment::from(out_1),
        );
        // Prover makes a proof using prover_cs
        let prover_gens = Generators::new(
            PedersenGenerators::default(),
            prover_cs.multiplications_count(),
            1,
        );
        let (proof, verifier_input) = prover_cs.prove(&v_blinding, &prover_gens, &mut rng)?;

        // Verifier makes a `ConstraintSystem` instance representing a shuffle gadget
        let mut verifier_transcript = Transcript::new(b"ShuffleTest");
        let (mut verifier_cs, _committed_variables) =
            ConstraintSystem::verifier_new(&mut verifier_transcript, commitments);
        // Verifier allocates variables and adds constraints to the constraint system
        shuffle_cs(
            &mut verifier_cs,
            Assignment::Missing(),
            Assignment::Missing(),
            Assignment::Missing(),
            Assignment::Missing(),
        );
        // Verifier verifies proof
        let verifier_gens = Generators::new(
            PedersenGenerators::default(),
            verifier_cs.multiplications_count(),
            1,
        );
        Ok(verifier_cs.verify(&proof, &verifier_input, &verifier_gens, &mut rng)?)
    }

    fn shuffle_cs(
        cs: &mut ConstraintSystem,
        in_0: Assignment,
        in_1: Assignment,
        out_0: Assignment,
        out_1: Assignment,
    ) {
        let (in_0_var, in_1_var) = cs.assign_uncommitted(in_0, in_1);
        let (out_0_var, out_1_var) = cs.assign_uncommitted(out_0, out_1);

        Shuffle::fill_cs(
            cs,
            (in_0_var, in_0),
            (in_1_var, in_1),
            (out_0_var, out_0),
            (out_1_var, out_1),
        );
    }

    #[test]
    fn merge_gadget() {
        let peso = 66;
        let peso_tag = 77;
        let yuan = 88;
        let yuan_tag = 99;

        // no merge, different asset types
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (6, yuan, yuan_tag),
                (3, peso, peso_tag),
                (6, yuan, yuan_tag),
            ).is_ok()
        );
        // merge, same asset types
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (6, peso, peso_tag),
                (0, peso, peso_tag),
                (9, peso, peso_tag),
            ).is_ok()
        );
        // merge, zero value is different asset type
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (6, peso, peso_tag),
                (0, yuan, yuan_tag),
                (9, peso, peso_tag),
            ).is_ok()
        );
        // no merge, same asset types
        assert!(
            merge_helper(
                (6, peso, peso_tag),
                (6, peso, peso_tag),
                (6, peso, peso_tag),
                (6, peso, peso_tag),
            ).is_ok()
        );
        // error when merging different asset types
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (3, yuan, yuan_tag),
                (0, peso, peso_tag),
                (6, yuan, yuan_tag),
            ).is_err()
        );
        // error when not merging, but asset type changes
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (3, yuan, yuan_tag),
                (3, peso, peso_tag),
                (3, peso, peso_tag),
            ).is_err()
        );
        // error when creating more value (same asset types)
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (3, peso, peso_tag),
                (3, peso, peso_tag),
                (6, peso, peso_tag),
            ).is_err()
        );
        // error when creating more value (different asset types)
        assert!(
            merge_helper(
                (3, peso, peso_tag),
                (3, yuan, yuan_tag),
                (3, peso, peso_tag),
                (6, yuan, yuan_tag),
            ).is_err()
        );
    }

    fn merge_helper(
        A: (u64, u64, u64),
        B: (u64, u64, u64),
        C: (u64, u64, u64),
        D: (u64, u64, u64),
    ) -> Result<(), R1CSError> {
        let mut rng = OsRng::new().unwrap();

        // Prover makes a `ConstraintSystem` instance representing a merge gadget
        // v and v_blinding emptpy because we are only testing low-level variable constraints
        let v = vec![];
        let v_blinding = vec![];
        let mut prover_transcript = Transcript::new(b"MergeTest");
        let (mut prover_cs, _committed_variables, commitments) = ConstraintSystem::prover_new(
            &mut prover_transcript,
            v,
            v_blinding.clone(),
            PedersenGenerators::default(),
        );
        // Prover allocates variables and adds constraints to the constraint system
        let (A_q, B_q) = prover_cs.assign_uncommitted(Assignment::from(A.0), Assignment::from(B.0));
        let (C_q, D_q) = prover_cs.assign_uncommitted(Assignment::from(C.0), Assignment::from(D.0));
        let (A_a, B_a) = prover_cs.assign_uncommitted(Assignment::from(A.1), Assignment::from(B.1));
        let (C_a, D_a) = prover_cs.assign_uncommitted(Assignment::from(C.1), Assignment::from(D.1));
        let (A_t, B_t) = prover_cs.assign_uncommitted(Assignment::from(A.2), Assignment::from(B.2));
        let (C_t, D_t) = prover_cs.assign_uncommitted(Assignment::from(C.2), Assignment::from(D.2));
        let A = Value {
            q: (A_q, Assignment::from(A.0)),
            a: (A_a, Assignment::from(A.1)),
            t: (A_t, Assignment::from(A.2)),
        };
        let B = Value {
            q: (B_q, Assignment::from(B.0)),
            a: (B_a, Assignment::from(B.1)),
            t: (B_t, Assignment::from(B.2)),
        };
        let C = Value {
            q: (C_q, Assignment::from(C.0)),
            a: (C_a, Assignment::from(C.1)),
            t: (C_t, Assignment::from(C.2)),
        };
        let D = Value {
            q: (D_q, Assignment::from(D.0)),
            a: (D_a, Assignment::from(D.1)),
            t: (D_t, Assignment::from(D.2)),
        };
        Merge::fill_cs(&mut prover_cs, A, B, C, D);
        // Prover makes a proof using prover_cs
        let prover_gens = Generators::new(
            PedersenGenerators::default(),
            prover_cs.multiplications_count(),
            1,
        );
        let (proof, verifier_input) = prover_cs.prove(&v_blinding, &prover_gens, &mut rng)?;

        // Verifier makes a `ConstraintSystem` instance representing a merge gadget
        let mut verifier_transcript = Transcript::new(b"MergeTest");
        let (mut verifier_cs, _committed_variables) =
            ConstraintSystem::verifier_new(&mut verifier_transcript, commitments);
        // Verifier allocates variables and adds constraints to the constraint system
        let (A_q, B_q) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let (C_q, D_q) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let (A_a, B_a) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let (C_a, D_a) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let (A_t, B_t) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let (C_t, D_t) =
            verifier_cs.assign_uncommitted(Assignment::Missing(), Assignment::Missing());
        let A = Value {
            q: (A_q, Assignment::Missing()),
            a: (A_a, Assignment::Missing()),
            t: (A_t, Assignment::Missing()),
        };
        let B = Value {
            q: (B_q, Assignment::Missing()),
            a: (B_a, Assignment::Missing()),
            t: (B_t, Assignment::Missing()),
        };
        let C = Value {
            q: (C_q, Assignment::Missing()),
            a: (C_a, Assignment::Missing()),
            t: (C_t, Assignment::Missing()),
        };
        let D = Value {
            q: (D_q, Assignment::Missing()),
            a: (D_a, Assignment::Missing()),
            t: (D_t, Assignment::Missing()),
        };
        Merge::fill_cs(&mut verifier_cs, A, B, C, D);
        // Verifier verifies proof
        let verifier_gens = Generators::new(
            PedersenGenerators::default(),
            verifier_cs.multiplications_count(),
            1,
        );
        Ok(verifier_cs.verify(&proof, &verifier_input, &verifier_gens, &mut rng)?)
    }
}
