use std::{collections::HashMap, num::NonZeroUsize, sync::{atomic::AtomicUsize, Mutex}};

use atomic_float::AtomicF64;
use lazy_static::lazy_static;

use crate::{color_float3::ColorFloat3, vec2::FixedPoint};

pub const HALF_PROBABILITY_RAW_SCORE: AtomicF64 = AtomicF64::new(10.0);
pub const HALF_PROBABILITY_OPPORTUNITY_COST: AtomicF64 = AtomicF64::new(0.5);

// pub const MAX_TRACES_PER_ITERATION: usize = 4; // Maximum number of traces per iteration
pub const MAX_GENERATION_ATTEMPTS: AtomicUsize = AtomicUsize::new(4); // Maximum number of attempts to generate a trace

pub const FIRST_ITERATION_PROBABILITY: AtomicF64 = AtomicF64::new(0.5); // Probability for the first iteration
pub const SECOND_ITERATION_PROBABILITY: AtomicF64 = AtomicF64::new(0.25); // Probability for the second iteration

pub const FIRST_ITERATION_NUM_TRACES: AtomicUsize = AtomicUsize::new(1); // this is immutable, just for consistency

pub const SECOND_ITERATION_NUM_TRACES: AtomicUsize = AtomicUsize::new(3);

// pub const BLOCK_THREAD: bool = true; // Whether to block the thread when waiting for a trace to be generated
// pub const DISPLAY_ASTAR: bool = true; // Whether to display the A* search process

pub const ASTAR_MAX_EXPANSIONS: AtomicUsize = AtomicUsize::new(2000); // Maximum number of trials to find a trace

pub const VIA_COST: AtomicF64 = AtomicF64::new(5.0); // Cost of placing a via

pub const NUM_TOP_RANKED_TO_TRY: AtomicUsize = AtomicUsize::new(3); // Number of top-ranked traces to try fixing in each iteration

pub const SAMPLE_ITERATIONS: AtomicUsize = AtomicUsize::new(2);

pub const UPDATE_PROBA_SKIP_STRIDE: AtomicUsize = AtomicUsize::new(2); // Number of traces to skip when updating the probability

pub const LAYER_TO_TRACE_COLOR: [ColorFloat3; 6] = [
    ColorFloat3::new(1.0, 0.0, 0.0), // Red for front layer
    ColorFloat3::new(0.0, 0.0, 1.0), // Blue for back layer
    ColorFloat3::new(1.0, 1.0, 0.0), // Yellow for top layer
    ColorFloat3::new(0.0, 1.0, 0.0), // Green for bottom layer
    ColorFloat3::new(1.0, 0.0, 1.0), // Magenta for layer 5
    ColorFloat3::new(0.0, 1.0, 1.0), // Cyan for layer 6
];

pub fn astar_stride_from_raw(raw_stride: f64) -> FixedPoint{
    let mut result = FixedPoint::from_num(raw_stride);
    let result_bits = result.to_bits();
    if result_bits & 1 == 1{
        result += FixedPoint::DELTA;
    }
    assert!(result.to_bits() & 1 == 0, "A* stride must be even");
    result
}

lazy_static! {
    pub static ref SAMPLE_CNT: AtomicUsize = AtomicUsize::new(0); // Global counter for the number of samples taken
    pub static ref ASTAR_STRIDE: Mutex<FixedPoint> = {
        let raw_stride: f64 = 2.00;
        Mutex::new(astar_stride_from_raw(raw_stride))
    }; // A* search stride
    // pub static ref NEXT_ITERATION_TO_REMAINING_PROBABILITY: HashMap<NonZeroUsize, f64> = vec![
    //     (
    //         NonZeroUsize::new(1).unwrap(),
    //         1.0
    //     ),
    //     (
    //         NonZeroUsize::new(2).unwrap(),
    //         1.0 - FIRST_ITERATION_SUM_PROBABILITY
    //     ),
    //     (
    //         NonZeroUsize::new(3).unwrap(),
    //         1.0 - FIRST_ITERATION_SUM_PROBABILITY - SECOND_ITERATION_SUM_PROBABILITY
    //     ),
    //     (
    //         NonZeroUsize::new(4).unwrap(),
    //         1.0 - FIRST_ITERATION_SUM_PROBABILITY
    //             - SECOND_ITERATION_SUM_PROBABILITY
    //             - THIRD_ITERATION_SUM_PROBABILITY
    //     ),
    //     (
    //         NonZeroUsize::new(5).unwrap(),
    //         1.0 - FIRST_ITERATION_SUM_PROBABILITY
    //             - SECOND_ITERATION_SUM_PROBABILITY
    //             - THIRD_ITERATION_SUM_PROBABILITY
    //             - FOURTH_ITERATION_SUM_PROBABILITY
    //     ),
    // ]
    // .into_iter()
    // .collect();
    // pub static ref ITERATION_TO_NUM_TRACES: HashMap<NonZeroUsize, usize> = vec![
    //     (NonZeroUsize::new(1).unwrap(), FIRST_ITERATION_NUM_TRACES),
    //     (NonZeroUsize::new(2).unwrap(), SECOND_ITERATION_NUM_TRACES),
    // ]
    // .into_iter()
    // .collect();
}
