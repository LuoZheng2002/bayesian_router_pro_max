use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    rc::Rc,
    sync::{Arc, Mutex, atomic::Ordering},
};

use fixed::traits::Fixed;
use ordered_float::NotNan;

use crate::{command_flags::{CommandFlag, TARGET_COMMAND_LEVEL}, display_injection::DisplayInjection, post_process::optimize_path};
use crate::{
    quad_tree::QuadTreeNode,
};

use shared::{
    binary_heap_item::BinaryHeapItem,
    collider::{BorderCollider, Collider},
    hyperparameters::{ASTAR_STRIDE, ASTAR_MAX_EXPANSIONS, VIA_COST},
    octile_distance::octile_distance_fixed,
    pad::PadLayer,
    pcb_render_model::{
        self, PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel,
    },
    prim_shape::{CircleShape, PrimShape, RectangleShape},
    trace_path::{
        self, AStarNodeDirection, Direction, TraceAnchor, TraceAnchors, TracePath, TraceSegment, Via
    },
    vec2::{FixedPoint, FixedVec2, FloatVec2},
};

pub struct AStarModel {
    pub width: f32,
    pub height: f32,
    pub center: FloatVec2,
    pub obstacle_shapes: Rc<HashMap<usize, Vec<PrimShape>>>,
    pub obstacle_clearance_shapes: Rc<HashMap<usize, Vec<PrimShape>>>,
    pub obstacle_colliders: Rc<HashMap<usize, QuadTreeNode>>,
    pub obstacle_clearance_colliders: Rc<HashMap<usize, QuadTreeNode>>,
    pub start: FixedVec2,
    pub end: FixedVec2,
    pub start_layers: PadLayer,
    pub end_layers: PadLayer,
    pub num_layers: usize,
    pub trace_width: f32,
    pub trace_clearance: f32,
    pub via_diameter: f32,
    pub border_colliders_cache: RefCell<Option<Rc<Vec<Collider>>>>,
    pub border_shapes_cache: RefCell<Option<Rc<Vec<PrimShape>>>>,
}

impl AStarModel {
    fn in_layers(&self, layer_index: usize, layers: PadLayer) -> bool {
        assert!(
            layer_index < self.num_layers,
            "Layer index out of bounds: {}, num_layers: {}",
            layer_index,
            self.num_layers
        );
        match layers {
            PadLayer::All => true,
            PadLayer::Front => layer_index == 0,
            PadLayer::Back => {
                assert!(
                    self.num_layers > 1,
                    "Number of layers must be greater than 1"
                );
                layer_index == self.num_layers - 1
            }
        }
    }
    pub fn calculate_border_colliders(
        width: f32,
        height: f32,
        center: FloatVec2,
    ) -> Rc<Vec<Collider>> {
        let left_border = BorderCollider {
            point_on_border: FloatVec2::new(center.x - width / 2.0, 0.0),
            normal: FloatVec2::new(-1.0, 0.0),
        };
        let right_border = BorderCollider {
            point_on_border: FloatVec2::new(center.x + width / 2.0, 0.0),
            normal: FloatVec2::new(1.0, 0.0),
        };
        let top_border = BorderCollider {
            point_on_border: FloatVec2::new(0.0, center.y + height / 2.0),
            normal: FloatVec2::new(0.0, 1.0),
        };
        let bottom_border = BorderCollider {
            point_on_border: FloatVec2::new(0.0, center.y - height / 2.0),
            normal: FloatVec2::new(0.0, -1.0),
        };
        Rc::new(vec![
            Collider::Border(left_border),
            Collider::Border(right_border),
            Collider::Border(top_border),
            Collider::Border(bottom_border),
        ])
    }
    fn get_border_colliders(&self) -> Rc<Vec<Collider>> {
        if let Some(border_shapes) = self.border_colliders_cache.borrow().as_ref() {
            return border_shapes.clone();
        }
        let border_colliders =
            Self::calculate_border_colliders(self.width, self.height, self.center);
        *self.border_colliders_cache.borrow_mut() = Some(border_colliders.clone());
        border_colliders
    }
    fn get_border_shapes(&self) -> Rc<Vec<PrimShape>> {
        if let Some(border_shapes) = self.border_shapes_cache.borrow().as_ref() {
            return border_shapes.clone();
        }
        let top_left = FloatVec2::new(
            self.center.x - self.width / 2.0,
            self.center.y + self.height / 2.0,
        );
        let top_right = FloatVec2::new(
            self.center.x + self.width / 2.0,
            self.center.y + self.height / 2.0,
        );
        let bottom_left = FloatVec2::new(
            self.center.x - self.width / 2.0,
            self.center.y - self.height / 2.0,
        );
        let bottom_right = FloatVec2::new(
            self.center.x + self.width / 2.0,
            self.center.y - self.height / 2.0,
        );
        let left_border_shape = PrimShape::Line(shared::prim_shape::Line {
            start: top_left,
            end: bottom_left,
        });
        let right_border_shape = PrimShape::Line(shared::prim_shape::Line {
            start: top_right,
            end: bottom_right,
        });
        let top_border_shape = PrimShape::Line(shared::prim_shape::Line {
            start: top_left,
            end: top_right,
        });
        let bottom_border_shape = PrimShape::Line(shared::prim_shape::Line {
            start: bottom_left,
            end: bottom_right,
        });
        let border_shapes: Rc<Vec<PrimShape>> = Rc::new(vec![
            left_border_shape,
            right_border_shape,
            top_border_shape,
            bottom_border_shape,
        ]);
        *self.border_shapes_cache.borrow_mut() = Some(border_shapes.clone());
        border_shapes
    }

    fn collides_with_border<'a, I>(&self, colliders: I) -> bool
    where
        I: Iterator<Item = &'a Collider> + Clone,
    {
        // the allowed region is between (-width/2, -height/2) and (width/2, height/2)
        // create four overlapping rectangles that encapsulate the allowed region
        // the margin is sufficiently large
        let border_colliders = self.get_border_colliders();
        Self::check_collision_between_two_sets(colliders, border_colliders.iter())
    }

    pub fn clamp_by_collision(
        &self,
        start_pos: FixedVec2,
        end_pos: FixedVec2,
        layer: usize,
    ) -> Option<FixedVec2> {
        assert!(Direction::is_two_points_valid_direction(start_pos, end_pos));
        if self.check_collision_for_trace(
            start_pos,
            end_pos,
            self.trace_width,
            self.trace_clearance,
            layer,
        ) {
            self.binary_approach_to_obstacles(start_pos, end_pos, layer)
        } else {
            Some(end_pos)
        }
    }
    /// outputs shapes and clearance shapes
    fn construct_trace_segment(
        start_position: FixedVec2,
        end_position: FixedVec2,
        trace_width: f32,
        trace_clearance: f32,
    ) -> TraceSegment {
        assert_ne!(
            start_position, end_position,
            "Start and end positions should not be the same"
        );
        TraceSegment {
            start: start_position,
            end: end_position,
            width: trace_width,
            clearance: trace_clearance,
            layer: 0, // layer is not used in this function, but we need to provide it
        }
    }

    fn check_collision_between_two_sets<'a, 'b, I1, I2>(colliders1: I1, colliders2: I2) -> bool
    where
        I1: Iterator<Item = &'a Collider> + Clone,
        I2: Iterator<Item = &'b Collider> + Clone,
    {
        for collider1 in colliders1 {
            for collider2 in colliders2.clone() {
                if collider1.collides_with(collider2) {
                    return true;
                }
            }
        }
        false
    }

    fn check_collision_for_trace(
        &self,
        start_position: FixedVec2,
        end_position: FixedVec2,
        trace_width: f32,
        trace_clearance: f32,
        layer: usize,
    ) -> bool {
        assert_ne!(
            start_position, end_position,
            "Start and end positions should not be the same"
        );
        let trace_segment = Self::construct_trace_segment(
            start_position,
            end_position,
            trace_width,
            trace_clearance,
        );
        let trace_segment_colliders = trace_segment.to_colliders();
        let trace_segment_clearance_colliders = trace_segment.to_clearance_colliders();
        let obstacle_colliders = self.obstacle_colliders.get(&layer).unwrap();
        let obstacle_clearance_colliders = self.obstacle_clearance_colliders.get(&layer).unwrap();
        if obstacle_colliders.collides_with_set(trace_segment_clearance_colliders.iter()) {
            return true; // collision with an obstacle clearance shape
        }
        if obstacle_clearance_colliders.collides_with_set(trace_segment_colliders.iter()) {
            return true; // collision with an obstacle
        }
        if self.collides_with_border(trace_segment_colliders.iter()) {
            return true; // collision with the border
        }
        false // no collision
    }
    fn check_collision_for_via(
        &self,
        position: FixedVec2,
        via_diameter: f32,
        clearance: f32,
        layer: usize,
    ) -> bool {
        let shape = PrimShape::Circle(CircleShape {
            position: position.to_float(),
            diameter: via_diameter,
        });
        let clearance_shape = PrimShape::Circle(CircleShape {
            position: position.to_float(),
            diameter: via_diameter + clearance * 2.0,
        });
        let collider = Collider::from_prim_shape(&shape);
        let clearance_collider = Collider::from_prim_shape(&clearance_shape);
        let obstacle_colliders = self.obstacle_colliders.get(&layer).unwrap();
        let obstacle_clearance_colliders = self.obstacle_clearance_colliders.get(&layer).unwrap();
        if obstacle_clearance_colliders.collides_with(&collider) {
            return true; // collision with an obstacle clearance shape
        }
        if obstacle_colliders.collides_with(&clearance_collider) {
            return true; // collision with an obstacle
        }
        if self.collides_with_border(std::iter::once(&collider)) {
            return true; // collision with the border
        }
        false // no collision
    }

    fn is_grid_point(&self, position: &FixedVec2, astar_stride: FixedPoint) -> bool {
        position.x % astar_stride == FixedPoint::ZERO
            && position.y % astar_stride == FixedPoint::ZERO
    }

    fn clamp_down(value: FixedPoint, astar_stride: FixedPoint) -> FixedPoint {
        if value > FixedPoint::ZERO {
            ((value - FixedPoint::DELTA) / astar_stride).floor() * astar_stride
        } else {
            (value / astar_stride - FixedPoint::DELTA).floor() * astar_stride
        }
    }
    fn clamp_up(value: FixedPoint, astar_stride: FixedPoint) -> FixedPoint {
        if value >= FixedPoint::ZERO {
            (value / astar_stride + FixedPoint::DELTA).ceil() * astar_stride
        } else {
            ((value + FixedPoint::DELTA) / astar_stride).ceil() * astar_stride
        }
    }

    /// outputs the pairs of direction and the grid point that the direction leads to
    /// not implemented the collision check yet
    fn directions_to_grid_points(&self, position: FixedVec2, astar_stride: FixedPoint) -> Vec<(Direction, FixedVec2)> {
        let mut result: Vec<(Direction, FixedVec2)> = Vec::new();
        // horizontal directions
        if position.y.rem_euclid(astar_stride) == FixedPoint::ZERO {
            // left
            let left_grid_point_x = Self::clamp_down(position.x, astar_stride);
            let right_grid_point_x = Self::clamp_up(position.x, astar_stride);
            let left_grid_point = FixedVec2::new(left_grid_point_x, position.y);
            let right_grid_point = FixedVec2::new(right_grid_point_x, position.y);
            assert_ne!(
                position, left_grid_point,
                "Left grid point should not be the same as position"
            );
            assert_ne!(
                position, right_grid_point,
                "Right grid point should not be the same as position"
            );
            assert!(Direction::is_two_points_valid_direction(
                position,
                left_grid_point
            ));
            assert!(Direction::is_two_points_valid_direction(
                position,
                right_grid_point
            ));
            result.push((Direction::Left, left_grid_point));
            result.push((Direction::Right, right_grid_point));
        }
        // vertical directions
        if position.x.rem_euclid(astar_stride) == FixedPoint::ZERO {
            // up
            let up_grid_point_y = Self::clamp_up(position.y, astar_stride);
            let down_grid_point_y = Self::clamp_down(position.y, astar_stride);
            let up_grid_point = FixedVec2::new(position.x, up_grid_point_y);
            let down_grid_point = FixedVec2::new(position.x, down_grid_point_y);
            assert_ne!(
                position, up_grid_point,
                "Up grid point should not be the same as position"
            );
            assert_ne!(
                position, down_grid_point,
                "Down grid point should not be the same as position"
            );
            assert!(Direction::is_two_points_valid_direction(
                position,
                up_grid_point
            ));
            assert!(Direction::is_two_points_valid_direction(
                position,
                down_grid_point
            ));
            result.push((Direction::Up, up_grid_point));
            result.push((Direction::Down, down_grid_point));
        }
        // top left to bottom right diagonal
        if (position.x + position.y).rem_euclid(astar_stride) == FixedPoint::ZERO {
            let top_left_grid_point =
                FixedVec2::new(Self::clamp_down(position.x, astar_stride), Self::clamp_up(position.y, astar_stride));
            let bottom_right_grid_point =
                FixedVec2::new(Self::clamp_up(position.x, astar_stride), Self::clamp_down(position.y, astar_stride));
            assert_ne!(
                position, top_left_grid_point,
                "Top left grid point should not be the same as position"
            );
            assert_ne!(
                position, bottom_right_grid_point,
                "Bottom right grid point should not be the same as position"
            );
            // assert!(Direction::is_two_points_valid_direction(position, top_left_grid_point),
            //     "old position: {:?}, new position: {:?}, dx: {}, dy: {}, direction: TopLeft",
            //     position, top_left_grid_point, top_left_grid_point.x - position.x, top_left_grid_point.y - position.y);
            if !Direction::is_two_points_valid_direction(position, top_left_grid_point) {
                println!(
                    "Invalid TopLeft direction: old position: {:?}, new position: {:?}, dx: {}, dy: {}, direction: TopLeft",
                    position,
                    top_left_grid_point,
                    top_left_grid_point.x - position.x,
                    top_left_grid_point.y - position.y
                );
                println!(
                    "x % ASTAR_STRIDE: {}, y % ASTAR_STRIDE: {}",
                    position.x.rem_euclid(astar_stride).to_bits(),
                    position.y.rem_euclid(astar_stride).to_bits()
                );
                println!("ASTAR_STRIDE: {}", astar_stride.to_bits());
                panic!("Invalid TopLeft direction");
            }
            assert!(Direction::is_two_points_valid_direction(
                position,
                bottom_right_grid_point
            ));
            result.push((Direction::TopLeft, top_left_grid_point));
            result.push((Direction::BottomRight, bottom_right_grid_point));
        }
        // top right to bottom left diagonal
        if (position.x - position.y).rem_euclid(astar_stride) == FixedPoint::ZERO {
            let top_right_grid_point =
                FixedVec2::new(Self::clamp_up(position.x, astar_stride), Self::clamp_up(position.y, astar_stride));
            let bottom_left_grid_point =
                FixedVec2::new(Self::clamp_down(position.x, astar_stride), Self::clamp_down(position.y, astar_stride));
            assert_ne!(
                position, top_right_grid_point,
                "Top right grid point should not be the same as position"
            );
            assert_ne!(
                position, bottom_left_grid_point,
                "Bottom left grid point should not be the same as position"
            );
            assert!(Direction::is_two_points_valid_direction(
                position,
                top_right_grid_point
            ));
            assert!(Direction::is_two_points_valid_direction(
                position,
                bottom_left_grid_point
            ));
            result.push((Direction::TopRight, top_right_grid_point));
            result.push((Direction::BottomLeft, bottom_left_grid_point));
        }
        result
    }
    fn radial_directions_wrt_obstacles(
        &self,
        position: &FixedVec2,
        layer: usize,
    ) -> Vec<Direction> {
        let mut directions: Vec<Direction> = Vec::new();
        let mut collides_at_direction: HashMap<Direction, bool> = HashMap::new();
        let twice_delta = FixedPoint::DELTA * 2;
        for direction in Direction::all_directions() {
            let end_position = *position + direction.to_fixed_vec2(twice_delta);
            assert_ne!(*position, end_position, "assert 1");
            let collides = self.check_collision_for_trace(
                *position,
                end_position,
                self.trace_width,
                self.trace_clearance,
                layer,
            );
            collides_at_direction.insert(direction, collides);
        }
        let is_valid_radial_direction =
            |left_90_dir: Direction,
             left_45_dir: Direction,
             dir: Direction,
             right_45_dir: Direction,
             right_90_dir: Direction| {
                // check if the direction is valid, i.e., it is not a 45-degree direction
                // or it is a 45-degree direction but both left and right directions are not valid
                let left_blocked =
                    collides_at_direction[&left_90_dir] && collides_at_direction[&left_45_dir];
                let right_blocked =
                    collides_at_direction[&right_90_dir] && collides_at_direction[&right_45_dir];
                let front_blocked = collides_at_direction[&dir];
                !front_blocked && (left_blocked || right_blocked)
            };
        for direction in Direction::all_directions() {
            let left_90_dir = direction.left_90_dir();
            let left_45_dir = direction.left_45_dir();
            let right_45_dir = direction.right_45_dir();
            let right_90_dir = direction.right_90_dir();
            if is_valid_radial_direction(
                left_90_dir,
                left_45_dir,
                direction,
                right_45_dir,
                right_90_dir,
            ) {
                directions.push(direction);
            }
        }
        directions
    }
    /// 将浮动点移动到稍微好一点的点
    fn to_nearest_one_step_point(&self, position: &FixedVec2, direction: Direction, astar_stride: FixedPoint) -> FixedVec2 {
        let is_difference_even = (position.x - position.y).to_bits() % 2 == 0;
        assert!(
            is_difference_even,
            "The difference between x and y should be even, x:{}, y:{}, direction: {:?}",
            position.x, position.y, direction
        );
        // an odd odd point cannot move non-diagonally
        assert!(direction.is_diagonal() || !position.is_x_odd_y_odd());
        let result = match direction {
            Direction::Up => {
                let new_y = Self::clamp_up(position.y, astar_stride);
                FixedVec2::new(position.x, new_y)
            }
            Direction::Down => {
                let new_y = Self::clamp_down(position.y, astar_stride);
                FixedVec2::new(position.x, new_y)
            }
            Direction::Left => {
                let new_x = Self::clamp_down(position.x, astar_stride);
                FixedVec2::new(new_x, position.y)
            }
            Direction::Right => {
                let new_x = Self::clamp_up(position.x, astar_stride);
                FixedVec2::new(new_x, position.y)
            }
            Direction::TopLeft => {
                // 左下到右上的线
                let current_difference = position.y - position.x;
                // new_position.y - new_position.x = target_difference
                // 左下到右上的线，往左上提
                let target_difference = Self::clamp_up(current_difference, astar_stride);
                // 往左上走，x和y的和不变
                let sum = position.y + position.x;
                // y - x = target_difference
                // y + x = sum
                // 求线性方程组
                let new_x = (sum - target_difference) / 2;
                let new_y = (sum + target_difference) / 2;
                FixedVec2::new(new_x, new_y)
            }
            Direction::BottomRight => {
                // 左下到右上的线
                let current_difference = position.y - position.x;
                // new_position.y - new_position.x = target_difference
                // 左下到右上的线，往右下按
                let target_difference = Self::clamp_down(current_difference, astar_stride);
                // 往左上走，x和y的和不变
                let sum = position.y + position.x;
                // y - x = target_difference
                // y + x = sum
                // 求线性方程组
                let new_x = (sum - target_difference) / 2;
                let new_y = (sum + target_difference) / 2;
                FixedVec2::new(new_x, new_y)
            }
            Direction::BottomLeft => {
                // 左上到右下的线
                let current_sum = position.x + position.y;
                // new_position.y + new_position.x = target_difference
                // 左上到右下的线， 往左下按
                let target_sum = Self::clamp_down(current_sum, astar_stride);
                // 往左下走，y和x的差不变
                let difference = position.y - position.x;
                // y - x = difference
                // y + x = target_sum
                // 求线性方程组
                let new_x = (target_sum - difference) / 2;
                let new_y = (target_sum + difference) / 2;
                FixedVec2::new(new_x, new_y)
            }
            Direction::TopRight => {
                // 左上到右下的线
                let current_sum = position.x + position.y;
                // new_position.y + new_position.x = target_difference
                // 左上到右下的线， 往右上按
                let target_sum = Self::clamp_up(current_sum, astar_stride);
                // 往左下走，y和x的差不变
                let difference = position.y - position.x;
                // y - x = difference
                // y + x = target_sum
                // 求线性方程组
                let new_x = (target_sum - difference) / 2;
                let new_y = (target_sum + difference) / 2;
                FixedVec2::new(new_x, new_y)
            }
        };
        assert!(Direction::is_two_points_valid_direction(*position, result));
        assert!(
            result.is_sum_even(),
            "Result position should be even, but got odd: {:?}",
            result
        );
        result
    }
    /// 判断当前点是否与目标点对齐，返回对齐的方向
    fn is_aligned_with_end(&self, position: FixedVec2, layer: usize) -> Option<Direction> {
        let end_layers = self
            .end_layers
            .get_iter(self.num_layers)
            .collect::<HashSet<_>>();
        if !end_layers.contains(&layer) {
            return None; // not aligned with end layer
        }
        assert_ne!(
            position, self.end,
            "调用该函数前应确保已经处理与end重合的情况"
        );
        match Direction::from_points(position, self.end) {
            Ok(direction) => match direction{
                Some(dir) => {
                    Some(dir)
                }
                None => None, // not aligned
            }
            Err(_) => None,
        }
    }
    /// line 1 is finite, line 2 is infinite
    fn line_intersection_infinite(
        &self,
        line1_start: FixedVec2,
        line1_end: FixedVec2,
        line2_start: FixedVec2,
        line2_end: FixedVec2,
    ) -> Option<FixedVec2> {
        assert!(line1_start.is_sum_even());
        assert!(line1_end.is_sum_even());
        assert!(line2_start.is_sum_even());
        let (dx1, dy1) = (line1_end.x - line1_start.x, line1_end.y - line1_start.y);
        let (dx2, dy2) = (line2_end.x - line2_start.x, line2_end.y - line2_start.y);

        // Line 1 coefficients: y = m1 * x + c1
        let (m1, c1) = if dx1 == 0 {
            (None, line1_start.x) // Vertical line: x = c1
        } else if dy1 == 0 {
            (Some(0), line1_start.y) // Horizontal line: y = c1
        } else if dy1 == dx1 {
            (Some(1), line1_start.y - line1_start.x) // 45°
        } else if dy1 == -dx1 {
            (Some(-1), line1_start.y + line1_start.x) // -45°
        } else {
            panic!(
                "Line 1 is not aligned with the grid, dx1: {}, dy1: {}",
                dx1, dy1
            );
        };

        // Line 2 coefficients
        let (m2, c2) = if dx2 == 0 {
            (None, line2_start.x)
        } else if dy2 == 0 {
            (Some(0), line2_start.y)
        } else if dy2 == dx2 {
            (Some(1), line2_start.y - line2_start.x)
        } else if dy2 == -dx2 {
            (Some(-1), line2_start.y + line2_start.x)
        } else {
            panic!(
                "Line 2 is not aligned with the grid, dx2: {}, dy2: {}",
                dx2, dy2
            );
        };

        // Intersection logic
        match (m1, m2) {
            (Some(m1), Some(m2)) => {
                assert_ne!(m1, m2, "Lines are parallel, no intersection");
                // m1 * x + c1 = m2 * x + c2 -> x = (c2 - c1) / (m1 - m2)
                let x = (c2 - c1) / (m1 - m2);
                let y = m1 * x + c1;
                // check boundary
                let x_min = FixedPoint::min(line1_start.x, line1_end.x);
                let x_max = FixedPoint::max(line1_start.x, line1_end.x);
                if x >= x_min && x <= x_max {
                    Some(FixedVec2 { x, y })
                } else {
                    None
                }
            }
            (None, Some(m2)) => {
                // Vertical line x = c1, plug into other line
                let x = c1;
                let y = m2 * x + c2;
                let y_min = FixedPoint::min(line1_start.y, line1_end.y);
                let y_max = FixedPoint::max(line1_start.y, line1_end.y);
                if y >= y_min && y <= y_max {
                    Some(FixedVec2 { x, y })
                } else {
                    None
                }
            }
            (Some(m1), None) => {
                let x = c2;
                let y = m1 * x + c1;
                let y_min = FixedPoint::min(line2_start.y, line2_end.y);
                let y_max = FixedPoint::max(line2_start.y, line2_end.y);
                if y >= y_min && y <= y_max {
                    Some(FixedVec2 { x, y })
                } else {
                    None
                }
            }
            (None, None) => {
                panic!("Both lines are vertical, which is not expected in this context")
            }
        }
    }

    /// 获取与end对齐的交点，还是给定方向和线段长度，判断是否有交叉
    /// allow for the node to be in a different layer from end, but will return none in this case
    fn get_intersection_with_end_alignments(
        &self,
        start_pos: FixedVec2,
        end_pos: FixedVec2,
        layer: usize,
    ) -> Option<FixedVec2> {
        let end_layers = self
            .end_layers
            .get_iter(self.num_layers)
            .collect::<HashSet<_>>();
        if !end_layers.contains(&layer) {
            return None; // not aligned with end layer
        }
        assert_ne!(
            start_pos, self.end,
            "调用该函数前应确保已经处理与end重合的情况"
        );
        assert!(
            self.is_aligned_with_end(start_pos, layer).is_none(),
            "调用该函数前应确保当前点不与end对齐"
        );
        assert!(start_pos.is_sum_even());
        assert!(end_pos.is_sum_even());

        let mut min_distance = FixedPoint::MAX;
        let mut best_intersection: Option<FixedVec2> = None;
        let current_direction = Direction::from_points(start_pos, end_pos).unwrap().unwrap();
        let mut end_directions = [
            current_direction.left_45_dir(),
            current_direction.right_45_dir(),
        ].into_iter().collect::<HashSet<_>>();
        end_directions.insert(Direction::Up);
        end_directions.insert(Direction::Down);
        end_directions.insert(Direction::Left);
        end_directions.insert(Direction::Right);

        for end_direction in end_directions {
            if end_direction == current_direction {
                continue; // skip the current direction
            }
            if end_direction == current_direction.opposite() {
                continue; // skip the opposite direction
            }
            // assert_ne!(end_direction, current_direction);
            // assert_ne!(end_direction, end_direction.opposite());
            if let Some(intersection) = self.line_intersection_infinite(
                start_pos,
                end_pos,
                self.end,
                self.end + end_direction.to_fixed_vec2(FixedPoint::DELTA),
            ) {
                // assert!(intersection.is_sum_even());
                let dx = intersection.x - start_pos.x;
                let dy = intersection.y - start_pos.y;
                let distance = FixedPoint::max(dx.abs(), dy.abs());
                assert!(distance != FixedPoint::ZERO, "Distance should not be zero");
                if distance < min_distance {
                    min_distance = distance;
                    best_intersection = Some(intersection);
                }
            }
        }
        best_intersection
    }

    fn binary_approach_to_obstacles(
        &self,
        start_position: FixedVec2,
        end_position: FixedVec2,
        layer: usize,
    ) -> Option<FixedVec2> {
        // println!("binary_approach_to_obstacles");
        let direction = Direction::from_points(start_position, end_position).unwrap().unwrap();
        let mut lower_bound = FixedPoint::from_num(0.0);
        let mut upper_bound = FixedPoint::max(
            (start_position.x - end_position.x).abs(),
            (start_position.y - end_position.y).abs(),
        );
        while lower_bound + FixedPoint::DELTA < upper_bound {
            let mid_length = (lower_bound + upper_bound) / 2;
            let temp_end = start_position + direction.to_fixed_vec2(mid_length);
            assert_ne!(start_position, temp_end, "assert 2");
            // let end_circle_clearance_shape = PrimShape::Circle(CircleShape {
            //     position: temp_end.to_float(),
            //     diameter: self.trace_width + self.trace_clearance * 2.0,
            // });
            if self.check_collision_for_trace(
                start_position,
                temp_end,
                self.trace_width,
                self.trace_clearance,
                layer,
            ) {
                upper_bound = mid_length; // collision found, search in the lower half
            } else {
                lower_bound = mid_length; // no collision, search in the upper half
            }
        }
        // assert_eq!(lower_bound, upper_bound, "Binary search should converge to a single point");
        assert!(
            (upper_bound - lower_bound).abs() <= FixedPoint::DELTA,
            "Binary search should converge to a single point"
        );
        let mut result_length = lower_bound;
        let end_position = start_position + direction.to_fixed_vec2(result_length);
        if !end_position.is_sum_even() || end_position.is_x_odd_y_odd() {
            result_length -= FixedPoint::DELTA; // ensure the result length is even
        }
        if result_length <= FixedPoint::ZERO {
            return None;
        }
        assert!(
            result_length > FixedPoint::ZERO,
            "Result length should be positive, but got: {}",
            result_length
        );
        let end_position = start_position + direction.to_fixed_vec2(result_length);
        assert!(
            end_position.is_sum_even(),
            "End position should be even, but got: {:?}",
            end_position
        );
        assert!(
            !end_position.is_x_odd_y_odd(),
            "End position should not be odd-odd, but got: {:?}",
            end_position
        );
        Some(end_position)
    }

    // 1. 整点/走一步到整点 -> 整点，或被障碍物挡住
    // 2. 走两步到整点+贴着障碍物 -> 对每个方向，走到最近的“走一步到整点”，或被障碍物挡住
    // 3. 是否align with end，如果是，并且align成功了的话，将end放入frontier

    // 拦住：网格边缘，align with end，障碍物
    // 障碍物优先，

    // 4. 浮空（走两步到整点+不贴障碍物）-> 选择任意的方向，走到“走一步到整点”，如果被障碍物挡住，选下一个方向；如果所有都被障碍物挡住，选择自己的方向并撞上障碍物

    // 同时考虑1和2和3
    // 如果满足1或2或3则不用4，如果1和2和3都失败则考虑4
    // 这些性质可以在expand的时候计算，不用存储
    // align with end也可以在expand的时候计算
    // 可能产生浮空的条件：起点，或是贴着墙走后不再贴着墙走

    // 伪代码：
    // current node从frontier中取出
    // current node设为visited
    // 判断1, 2, 3, 算出它们的expand的集合，然后合并（最多可能有8个方向，一个方向又最多可能有2个position）
    // 如果1, 2, 3都失败了（没有任何的expand），执行“4”的逻辑，必然会expand出来一个可能不怎么好的点
    // 将所有的expand的点放入frontier

    // shape

    fn astar_to_render_model(
        &self,
        frontier: &BinaryHeap<BinaryHeapItem<Reverse<NotNan<f64>>, Rc<AstarNode>>>,
    ) -> PcbRenderModel {
        let mut frontier_vec: Vec<BinaryHeapItem<Reverse<NotNan<f64>>, Rc<AstarNode>>> =
            frontier.clone().drain().collect();
        frontier_vec.reverse();
        let mut lowest_total_cost = f64::MAX;
        let mut highest_total_cost: f64 = 0.0;

        for item in frontier_vec.iter() {
            if item.key.0.into_inner() < lowest_total_cost {
                lowest_total_cost = item.key.0.into_inner();
            }
            if item.key.0.into_inner() > highest_total_cost {
                highest_total_cost = item.key.0.into_inner();
            }
        }
        let mut render_model = PcbRenderModel {
            width: self.width,
            height: self.height,
            center: self.center,
            trace_shape_renderables: Vec::new(),
            pad_shape_renderables: Vec::new(),
            other_shape_renderables: Vec::new(),
        };

        let obstacle_renderables = self
            .obstacle_shapes
            .iter()
            .flat_map(|(_, shapes)| shapes.iter())
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [0.7, 0.7, 0.7, 1.0], // gray obstacles
                }
            })
            .collect::<Vec<_>>();
        render_model
            .trace_shape_renderables
            .push(RenderableBatch(obstacle_renderables));
        let obstacle_clearance_renderables = self
            .obstacle_clearance_shapes
            .iter()
            .flat_map(|(_, shapes)| shapes.iter())
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [0.7, 0.7, 0.7, 0.5], // gray obstacle clearance
                }
            })
            .collect::<Vec<_>>();
        render_model
            .trace_shape_renderables
            .push(RenderableBatch(obstacle_clearance_renderables));
        // render border
        let border_renderables = self
            .get_border_shapes()
            .iter()
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [1.0, 0.0, 1.0, 0.5], // magenta border
                }
            })
            .collect::<Vec<_>>();
        render_model
            .other_shape_renderables
            .extend(border_renderables);

        let quad_tree_renderables = self
            .obstacle_colliders
            .iter()
            .flat_map(|(_, colliders)| colliders.to_outline_shapes());
        render_model
            .other_shape_renderables
            .extend(quad_tree_renderables.map(|shape| ShapeRenderable {
                shape,
                color: [0.0, 0.0, 1.0, 0.5], // blue quad tree colliders
            }));
        for item in frontier_vec.iter() {
            let BinaryHeapItem {
                key: total_cost,
                value: astar_node,
            } = item;
            let total_cost = total_cost.0.into_inner();
            assert!(
                total_cost >= lowest_total_cost,
                "Total cost should be greater than or equal to the lowest total cost"
            );
            assert!(
                total_cost <= highest_total_cost,
                "Total cost should be less than or equal to the highest total cost"
            );
            // let alpha = 1.0 - (0.2 + 0.8 * (total_cost - lowest_total_cost) / (highest_total_cost - lowest_total_cost));
            let alpha = if highest_total_cost > lowest_total_cost {
                1.0 - (0.2
                    + 0.8 * (total_cost - lowest_total_cost)
                        / (highest_total_cost - lowest_total_cost))
            } else {
                1.0 // if all costs are the same, use full opacity
            };
            let alpha = alpha.clamp(0.0, 1.0) as f32;
            assert!(
                alpha >= 0.0 && alpha <= 1.0,
                "Alpha should be between 0.0 and 1.0, get: {}",
                alpha
            );
            let color: [f32; 3] = [1.0 - alpha, alpha, 0.0]; // red to green gradient
            let renderables = astar_node.to_renderables(
                self.trace_width,
                self.trace_clearance,
                self.via_diameter,
                color,
            );
            render_model.trace_shape_renderables.extend(renderables);
        }
        // render the start and end nodes
        let start_renderable = ShapeRenderable {
            shape: PrimShape::Circle(CircleShape {
                position: self.start.to_float(),
                diameter: self.trace_width,
            }),
            color: [0.0, 0.0, 1.0, 1.0], // blue start node
        };
        let end_renderable = ShapeRenderable {
            shape: PrimShape::Circle(CircleShape {
                position: self.end.to_float(),
                diameter: self.trace_width,
            }),
            color: [0.0, 1.0, 0.0, 1.0], // green end node
        };
        render_model.other_shape_renderables.push(start_renderable);
        render_model.other_shape_renderables.push(end_renderable);
        render_model
    }

    fn display_when_necessary(
        &self,
        frontier: &BinaryHeap<BinaryHeapItem<Reverse<NotNan<f64>>, Rc<AstarNode>>>,
        command_flag: CommandFlag,
        display_injection: &mut DisplayInjection,
    ) {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            println!("A* search stopped by user");
            return;
        }
        // println!("Displaying A* frontier");
        let target_command_level = TARGET_COMMAND_LEVEL.load(Ordering::Relaxed);
        let task_command_level = command_flag.get_level();
        if target_command_level <= task_command_level {
            // Target level is less than task level
            // println!("Before blocking");
            let render_model = self.astar_to_render_model(frontier);
            while !(display_injection.can_submit_render_model)() {
                // Wait until we can submit the render model
            }            
            // println!("After blocking");
            (display_injection.submit_render_model)(render_model);
            (display_injection.block_until_signal)();
            //  println!("After signal");
        } else {
            if (display_injection.can_submit_render_model)() {
                // If we can submit the render model, do it
                let render_model = self.astar_to_render_model(frontier);
                (display_injection.submit_render_model)(render_model);
            }
        }        
    }
    fn final_trace_to_render_model(
        &self,
        trace: &TracePath,
    ) -> PcbRenderModel {
        let mut render_model = PcbRenderModel {
            width: self.width,
            height: self.height,
            center: self.center,
            trace_shape_renderables: Vec::new(),
            pad_shape_renderables: Vec::new(),
            other_shape_renderables: Vec::new(),
        };
        let obstacle_renderables = self
            .obstacle_shapes
            .iter()
            .flat_map(|(_, shapes)| shapes.iter())
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [0.7, 0.7, 0.7, 1.0], // gray obstacles
                }
            })
            .collect::<Vec<_>>();
        render_model
            .trace_shape_renderables
            .push(RenderableBatch(obstacle_renderables));
        let obstacle_clearance_renderables = self
            .obstacle_clearance_shapes
            .iter()
            .flat_map(|(_, shapes)| shapes.iter())
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [0.7, 0.7, 0.7, 0.5], // gray obstacle clearance
                }
            })
            .collect::<Vec<_>>();
        render_model
            .trace_shape_renderables
            .push(RenderableBatch(obstacle_clearance_renderables));
        // render border
        let border_renderables = self
            .get_border_shapes()
            .iter()
            .map(|shape| {
                ShapeRenderable {
                    shape: shape.clone(),
                    color: [1.0, 0.0, 1.0, 0.5], // magenta border
                }
            })
            .collect::<Vec<_>>();
        render_model
            .other_shape_renderables
            .extend(border_renderables);

        let quad_tree_renderables = self
            .obstacle_colliders
            .iter()
            .flat_map(|(_, colliders)| colliders.to_outline_shapes());
        render_model
            .other_shape_renderables
            .extend(quad_tree_renderables.map(|shape| ShapeRenderable {
                shape,
                color: [0.0, 0.0, 1.0, 0.5], // blue quad tree colliders
            }));
        // render the trace path
        let trace_renderables = trace.to_renderables([1.0, 0.5, 0.0, 1.0]);
        render_model.trace_shape_renderables.extend(trace_renderables);

        render_model
    }

    fn display_final_trace(&self,
        trace: &TracePath,
        command_flag: CommandFlag,
        display_injection: &mut DisplayInjection,
    ){
        let target_command_level = TARGET_COMMAND_LEVEL.load(Ordering::Relaxed);
        let task_command_level = command_flag.get_level();
        if target_command_level <= task_command_level {
            let render_model = self.final_trace_to_render_model(trace);
            while !(display_injection.can_submit_render_model)() {
                // wait until we can submit the render model
            }            
            (display_injection.submit_render_model)(render_model);
            (display_injection.block_until_signal)();        
        } else {
            if (display_injection.can_submit_render_model)() {
                // If we can submit the render model, do it
                let render_model = self.final_trace_to_render_model(trace);
                (display_injection.submit_render_model)(render_model);
            }
        }
    }

    pub fn run(
        &self,
        display_injection: &mut DisplayInjection,
    ) -> Result<AStarResult, String> {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            return Err("A* search stopped by user".to_string());
        }

        let astar_stride = {
            ASTAR_STRIDE.lock().unwrap().clone()
        };
        // println!("Running A*");
        // SAMPLE_CNT.fetch_add(1, Ordering::SeqCst);
        // println!("Sample count: {}", SAMPLE_CNT.load(Ordering::SeqCst));
        assert!(self.start.is_sum_even());
        assert!(self.end.is_sum_even());
        assert!(!self.start.is_x_odd_y_odd());
        assert!(!self.end.is_x_odd_y_odd());

        // frontier is a min heap
        let mut frontier: BinaryHeap<BinaryHeapItem<Reverse<NotNan<f64>>, Rc<AstarNode>>> =
            BinaryHeap::new();

        let start_estimated_cost =
            octile_distance_fixed(self.start, self.end);
        for layer in self.start_layers.get_iter(self.num_layers) {
            let start_node = AstarNode {
                position: self.start,
                layer,
                direction: AStarNodeDirection::None, // no direction for the start node
                actual_cost: 0.0,
                actual_length: 0.0, // no length for the start node
                estimated_cost: start_estimated_cost,
                total_cost: start_estimated_cost,
                prev_node: None, // no previous node for the start node
            };
            frontier.push(BinaryHeapItem {
                key: Reverse(NotNan::new(start_node.total_cost).unwrap()), // use Reverse to make it a min heap
                value: Rc::new(start_node),
            });
        }
        let mut visited: HashSet<AstarNodeKey> = HashSet::new();
        self.display_when_necessary( &frontier, CommandFlag::AstarInOut, display_injection); // display the initial state of the frontier

        let mut trial_count = 0;
        while !frontier.is_empty() {
            let item = frontier.pop().unwrap();

            let current_node = item.value.clone();
            if current_node.position == self.end {
                frontier.push(item); // push the current node back to the frontier, so that it can be displayed

                self.display_when_necessary(
                    &frontier,
                    CommandFlag::AstarInOut,
                    display_injection,
                ); // display the initial state of the frontier

                // Reached the end node, construct the trace path
                let trace_path = current_node.to_trace_path(
                    self.trace_width,
                    self.trace_clearance,
                    self.via_diameter,
                );
                let check_collision_for_trace =
                    |start: FixedVec2, end: FixedVec2, width: f32, clearance: f32, layer: usize| {
                        self.check_collision_for_trace(start, end, width, clearance, layer)
                    };
                // let check_collision_for_via =
                //     |position: FixedVec2, diameter: f32, clearance: f32, min_layer: usize, max_layer: usize| {
                //         for layer in min_layer..=max_layer {
                //             if self.check_collision_for_via(position, diameter, clearance, layer) {
                //                 return true; // collision found
                //             }
                //         }
                //         false
                //     };
                
                print!("Trace path directions:");
                for segment in trace_path.segments.iter() {
                    print!(" {:?}", segment.get_direction());                     
                }
                println!();
                self.display_final_trace(&trace_path, CommandFlag::AstarInOut, display_injection);         
                let trace_path = optimize_path(
                    &trace_path,
                    &check_collision_for_trace,
                    //  &check_collision_for_via,
                    self.trace_width,
                    self.trace_clearance,
                    self.via_diameter,
                );    
                println!("Finished one iteration of optimization");
                self.display_final_trace(&trace_path, CommandFlag::AstarInOut, display_injection);                
                return Ok(AStarResult { trace_path });
            }

            // move to the visited set
            let current_key = AstarNodeKey {
                position: current_node.position,
                layer: current_node.layer,
            };
            if visited.contains(&current_key) {
                continue; // already visited this node
            }
            // don't consider visited nodes as trials
            trial_count += 1;
            if trial_count > ASTAR_MAX_EXPANSIONS.load(Ordering::Relaxed) {
                // self.display_when_necessary(&frontier, CommandFlag::Auto, display_injection);
                return Err("A* search exceeded maximum trials".to_string());
            }
            visited.insert(current_key.clone());
            // expand

            // new:
            // hoist the closure out of the directions loop for the aligned_with_end condition
            let mut try_push_node_to_frontier =
                |direction: AStarNodeDirection, end_position: FixedVec2, end_layer: usize| {
                    assert!(
                        !matches!(direction, AStarNodeDirection::None),
                        "Direction should not be None"
                    );
                    assert!(
                        !end_position.is_x_odd_y_odd()
                            || !self.directions_to_grid_points(end_position, astar_stride).is_empty(),
                        "The end position should not be an odd-odd point if there are no directions to grid points"
                    );
                    let end_position_difference_even =
                        (end_position.x - end_position.y).to_bits() % 2 == 0;
                    assert!(
                        end_position_difference_even,
                        "The difference between x and y should be even, x:{}, y:{}, direction: {:?}",
                        end_position.x, end_position.y, direction
                    );

                    let astar_node_key = AstarNodeKey {
                        position: end_position,
                        layer: end_layer,
                    };
                    // check if the new position is already visited
                    if visited.contains(&astar_node_key) {
                        return;
                    }
                    // let length: f64 = (direction.to_fixed_vec2().length() * length).to_num();
                    let length: f64 = (end_position - current_node.position).length().to_num();
                    let via_cost = if let AStarNodeDirection::Vertical { .. } = direction {
                        VIA_COST.load(Ordering::Relaxed) // vertical movement has a via cost
                    } else {
                        0.0 // no via cost for planar movements
                    };
                    let actual_cost = current_node.actual_cost + length + via_cost;
                    let actual_length = current_node.actual_length + length;
                    let estimated_cost =
                        octile_distance_fixed(end_position, self.end);
                    let total_cost = actual_cost + estimated_cost;
                    let new_node = AstarNode {
                        position: end_position,
                        layer: end_layer,
                        direction,
                        actual_cost,
                        actual_length,
                        estimated_cost,
                        total_cost,
                        prev_node: Some(current_node.clone()), // link to the previous node
                    };
                    // push directly to the frontier
                    frontier.push(BinaryHeapItem {
                        key: Reverse(NotNan::new(new_node.total_cost).unwrap()), // use Reverse to make it a min heap
                        value: Rc::new(new_node),
                    });
                };

            assert!(
                !current_node.position.is_x_odd_y_odd()
                    || !self
                        .directions_to_grid_points(current_node.position, astar_stride)
                        .is_empty(),
                "The current position should not be an odd-odd point if there are no directions to grid points"
            );

            let mut current_node_handled = false;
            let mut condition_count = 0;

            // attempt a planar movement to reach the end
            let end_direction = self.is_aligned_with_end(current_node.position, current_node.layer);
            if let Some(end_direction) = end_direction {
                assert_ne!(current_node.position, self.end, "assert 3");
                if !self.check_collision_for_trace(
                    current_node.position,
                    self.end,
                    self.trace_width,
                    self.trace_clearance,
                    current_node.layer,
                ) {
                    // println!(
                    //     "is_aligned_with_end: ({}, {}) ({}, {})",
                    //     current_node.position.x, current_node.position.y, self.end.x, self.end.y
                    // );
                    assert!(
                        Direction::from_points(current_node.position, self.end).unwrap().unwrap()
                            == end_direction
                    );
                    condition_count = condition_count + 1;
                    try_push_node_to_frontier(
                        AStarNodeDirection::Planar(end_direction),
                        self.end,
                        current_node.layer,
                    );
                    // println!("Successfully pushed an end node to the frontier");
                }else{
                    // println!("Although a node is aligned with end, collision. Direction: {:?}", end_direction);
                }
            }

            // this will call try_push_node_to_frontier multiple times
            let mut try_place_vias = |position: FixedVec2,
                                      via_diameter: f32,
                                      clearance: f32,
                                      layer: usize| {
                if self.check_collision_for_via(position, via_diameter, clearance, layer) {
                    return;
                }
                for lower_layer in (0..layer).rev() {
                    if self.check_collision_for_via(position, via_diameter, clearance, lower_layer)
                    {
                        break;
                    }
                    try_push_node_to_frontier(
                        AStarNodeDirection::Vertical { from_layer: layer },
                        position,
                        lower_layer,
                    );
                }
                for upper_layer in (layer + 1)..self.num_layers {
                    if self.check_collision_for_via(position, via_diameter, clearance, upper_layer)
                    {
                        break;
                    }
                    try_push_node_to_frontier(
                        AStarNodeDirection::Vertical { from_layer: layer },
                        position,
                        upper_layer,
                    );
                }
            };
            // new: try place a via if the current node is at a grid point
            if self.is_grid_point(&current_node.position, astar_stride) {
                try_place_vias(
                    current_node.position,
                    self.via_diameter,
                    self.trace_clearance,
                    current_node.layer,
                );
            }

            // process grid points or one-step-to-grid-points
            // this is also planar
            let directions = self.directions_to_grid_points(current_node.position, astar_stride);
            assert!(
                directions.len() != 8 || self.is_grid_point(&current_node.position, astar_stride),
                "There should not be 8 directions to grid points if the current position is not a grid point"
            );
            for (direction, end_position) in directions {
                assert!(
                    Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                        == direction
                );
                current_node_handled = true;
                assert_ne!(current_node.position, end_position, "assert 5");

                let end_position = match self.clamp_by_collision(
                    current_node.position,
                    end_position,
                    current_node.layer,
                ) {
                    Some(pos) => pos,
                    None => continue, // if clamping fails, skip this direction
                };
                condition_count = condition_count + 1;
                assert!(
                    Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                        == direction
                );
                try_push_node_to_frontier(
                    AStarNodeDirection::Planar(direction),
                    end_position,
                    current_node.layer,
                );
                if let None = self.is_aligned_with_end(current_node.position, current_node.layer) {
                    if let Some(intersection) = self.get_intersection_with_end_alignments(
                        current_node.position,
                        end_position,
                        current_node.layer,
                    ) {
                        condition_count = condition_count + 1;
                        assert!(
                            Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                                == direction
                        );
                        try_push_node_to_frontier(
                            AStarNodeDirection::Planar(direction),
                            intersection,
                            current_node.layer,
                        );
                    }
                }
            }

            // process radial directions with respect to obstacles
            // this is also planar
            let radial_directions =
                self.radial_directions_wrt_obstacles(&current_node.position, current_node.layer);
            if !radial_directions.is_empty() {
                current_node_handled = true;
            }
            for direction in radial_directions {
                assert!(current_node.position.is_sum_even());
                if !direction.is_diagonal() && current_node.position.is_x_odd_y_odd() {
                    // 如果当前点是奇数点，且方向不是对角线方向，则不考虑该方向
                    continue;
                }
                let end_position =
                    self.to_nearest_one_step_point(&current_node.position, direction, astar_stride);
                assert_ne!(current_node.position, end_position, "assert 6");

                let end_position = match self.clamp_by_collision(
                    current_node.position,
                    end_position,
                    current_node.layer,
                ) {
                    Some(pos) => pos,
                    None => continue, // if clamping fails, skip this direction
                };
                condition_count = condition_count + 1;
                assert!(
                    Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                        == direction
                );
                try_push_node_to_frontier(
                    AStarNodeDirection::Planar(direction),
                    end_position,
                    current_node.layer,
                );
                if let None = self.is_aligned_with_end(current_node.position, current_node.layer) {
                    if let Some(intersection) = self.get_intersection_with_end_alignments(
                        current_node.position,
                        end_position,
                        current_node.layer,
                    ) {
                        condition_count = condition_count + 1;
                        assert!(
                            Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                                == direction
                        );
                        try_push_node_to_frontier(
                            AStarNodeDirection::Planar(direction),
                            intersection,
                            current_node.layer,
                        );
                    }
                }
            }

            if !current_node_handled {
                let mut found_point = false;
                for direction in Direction::all_directions() {
                    assert!(!current_node.position.is_x_odd_y_odd());
                    let end_position =
                        self.to_nearest_one_step_point(&current_node.position, direction, astar_stride);
                    assert_ne!(current_node.position, end_position, "assert 7");

                    if !self.check_collision_for_trace(
                        current_node.position,
                        end_position,
                        self.trace_width,
                        self.trace_clearance,
                        current_node.layer,
                    ) {
                        // println!("4: {}, {}", end_position.x, end_position.y);
                        condition_count = condition_count + 1;
                        assert!(
                            Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                                == direction
                        );
                        try_push_node_to_frontier(
                            AStarNodeDirection::Planar(direction),
                            end_position,
                            current_node.layer,
                        );
                        found_point = true;
                        break;
                    }
                }
                if !found_point {
                    // let self_direction = if !current_node.direction.is_none() {
                    //     current_node.direction.unwrap()
                    // } else {
                    //     Direction::Up
                    // };
                    let direction = match current_node.direction {
                        // Some(direction) => direction,
                        // None => Direction::Up, // default direction if not set
                        AStarNodeDirection::None => Direction::Up, // default direction if not set
                        AStarNodeDirection::Planar(direction) => direction,
                        AStarNodeDirection::Vertical { .. } => Direction::Up,
                    };
                    let end_position =
                        self.to_nearest_one_step_point(&current_node.position, direction, astar_stride);
                    if let Some(end_position) = self.clamp_by_collision(
                        current_node.position,
                        end_position,
                        current_node.layer,
                    ) {
                        // println!("4.1: {}, {}", temp_end.unwrap().x, temp_end.unwrap().y);
                        condition_count = condition_count + 1;
                        assert!(
                            Direction::from_points(current_node.position, end_position).unwrap().unwrap()
                                == direction
                        );
                        try_push_node_to_frontier(
                            AStarNodeDirection::Planar(direction),
                            end_position,
                            current_node.layer,
                        );
                    } else {
                        // remove the tried direction
                        let directions = Direction::all_directions()
                            .iter()
                            .filter(|&&d| d != direction && d != direction.opposite())
                            .cloned()
                            .collect::<Vec<_>>();
                        let mut found_point = false;
                        for direction in directions {
                            let end_position =
                                self.to_nearest_one_step_point(&current_node.position, direction, astar_stride);
                            if let Some(end_position) = self.clamp_by_collision(
                                current_node.position,
                                end_position,
                                current_node.layer,
                            ) {
                                // println!("4.2: {}, {}", end_position.x, end_position.y);
                                condition_count = condition_count + 1;
                                assert!(
                                    Direction::from_points(current_node.position, end_position)
                                        .unwrap().unwrap()
                                        == direction
                                );
                                try_push_node_to_frontier(
                                    AStarNodeDirection::Planar(direction),
                                    end_position,
                                    current_node.layer,
                                );
                                found_point = true;
                                break; // only try one direction
                            }
                        }
                        if !found_point {
                            println!(
                                "Warning: No valid point found for floating position {:?}",
                                current_node.position
                            );
                        }
                    }
                }
            }
            self.display_when_necessary(
                &frontier,
                CommandFlag::AstarFrontierOrUpdatePosterior,
                display_injection,
            ); // display the initial state of the frontier
        }
        // self.display_when_necessary(&frontier, CommandFlag::Auto, display_injection);
        Err("No path found".to_string()) // no path found
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AstarNodeKey {
    pub position: FixedVec2,
    pub layer: usize,
}

pub struct AstarNode {
    pub position: FixedVec2,
    pub layer: usize,
    pub direction: AStarNodeDirection, // the direction from the previous node to this node
    pub actual_cost: f64,              // the actual cost to reach this node from the start node
    pub actual_length: f64,
    pub estimated_cost: f64, // the estimated cost to reach the end node from this node
    pub total_cost: f64, // the total cost to reach this node from the start node, including the estimated cost to reach the end node
    pub prev_node: Option<Rc<AstarNode>>, // the previous node in the path, used for backtracking
}

impl AstarNode {
    fn is_direction_and_displacement_invariant(
        current_node: Rc<AstarNode>,
        prev_node: Option<Rc<AstarNode>>,
    ) -> bool {
        match current_node.direction {
            AStarNodeDirection::None => {
                if prev_node.is_some() {
                    println!("Warning: current node has no direction, but previous node exists");
                }

                prev_node.is_none() // no previous node
            }
            AStarNodeDirection::Planar(direction) => {
                let (prev_position, prev_layer) = match prev_node {
                    Some(node) => (node.position, node.layer),
                    None => return false, // if no previous node, return false
                };
                let calculated_direction =
                    Direction::from_points(prev_position, current_node.position);
                let calculated_direction = match calculated_direction {
                    Ok(dir) => match dir{
                        Some(dir) => dir,
                        None => {
                            return false; // if the direction cannot be calculated, return false
                        }
                    }
                    Err(_) => return false, // if the direction cannot be calculated, return false
                };
                if calculated_direction != direction {
                    println!(
                        "Warning: calculated direction {:?} does not match the expected direction {:?}",
                        calculated_direction, direction
                    );
                    println!(
                        "Current position: {:?}, Previous position: {:?}",
                        current_node.position, prev_position
                    );
                }
                if current_node.layer != prev_layer {
                    println!(
                        "Warning: current node layer {} does not match previous node layer {}",
                        current_node.layer, prev_layer
                    );
                }
                calculated_direction == direction && current_node.layer == prev_layer
            }
            AStarNodeDirection::Vertical { from_layer } => {
                let (prev_position, prev_layer) = match prev_node {
                    Some(node) => (node.position, node.layer),
                    None => {
                        println!(
                            "Warning: current node has no direction, but previous node exists"
                        );
                        return false;
                    }
                };
                prev_layer == from_layer
                    && current_node.layer != from_layer
                    && current_node.position == prev_position // the current layer should not be the same as the from_layer
            }
        }
    }
    pub fn to_trace_path(
        self: Rc<Self>,
        width: f32,
        clearance: f32,
        via_diameter: f32,
    ) -> TracePath {
        let mut current_node: Option<Rc<AstarNode>> = Some(self.clone());
        let mut next_node: Option<Rc<AstarNode>> = None;
        let mut anchors: Vec<TraceAnchor> = Vec::new(); // initializes with the end position
        let mut vias: Vec<Via> = Vec::new(); // initializes with the end position
        let mut pending_trace_anchor: Option<TraceAnchor> = None; // position, start, end
        while let Some(node) = &current_node {
            if let Some(next_node) = next_node {
                assert!(Self::is_direction_and_displacement_invariant(
                    next_node.clone(),
                    Some(node.clone())
                ));
            }
            pending_trace_anchor = if let Some(pending_anchor) = pending_trace_anchor {
                if node.position == pending_anchor.position {
                    Some(TraceAnchor {
                        position: node.position,
                        start_layer: node.layer,
                        end_layer: pending_anchor.end_layer,
                    })
                } else {
                    anchors.push(pending_anchor);
                    Some(TraceAnchor {
                        position: node.position,
                        start_layer: node.layer,
                        end_layer: node.layer,
                    })
                }
            } else {
                Some(TraceAnchor {
                    position: node.position,
                    start_layer: node.layer,
                    end_layer: node.layer,
                })
            };
            next_node = current_node.clone();
            current_node = node.prev_node.clone();
        }
        anchors.push(pending_trace_anchor.unwrap()); // push the last anchor
        let next_node = next_node.unwrap();
        assert!(Self::is_direction_and_displacement_invariant(
            next_node, None
        ));
        anchors.reverse(); // reverse the anchors to get the correct order
        let mut segments: Vec<TraceSegment> = Vec::new();
        for i in 0..anchors.len() - 1 {
            let start_anchor = &anchors[i];
            let end_anchor = &anchors[i + 1];
            assert!(
                start_anchor.end_layer == end_anchor.start_layer,
                "The end layer of the start anchor should match the start layer of the end anchor"
            );
            assert_ne!(
                start_anchor.position, end_anchor.position,
                "Start and end positions should not be the same"
            );
            let segment = TraceSegment {
                start: start_anchor.position,
                end: end_anchor.position,
                layer: start_anchor.end_layer,
                width,
                clearance,
            };
            segments.push(segment);
            if start_anchor.start_layer != start_anchor.end_layer {
                // if the start and end layers are different, we need to add a via
                let via = Via {
                    position: start_anchor.position,
                    clearance,
                    diameter: via_diameter,
                    min_layer: usize::min(start_anchor.start_layer, start_anchor.end_layer),
                    max_layer: usize::max(start_anchor.start_layer, start_anchor.end_layer),
                };
                vias.push(via);
            }
        }
        let anchors = TraceAnchors(anchors);
        assert!(
            self.estimated_cost == 0.0,
            "The estimated cost should be 0.0 for the trace path"
        );
        TracePath {
            anchors,
            segments,
            vias,
            total_length: self.actual_length,
        }
    }
    pub fn to_renderables(
        &self,
        width: f32,
        clearance: f32,
        via_diameter: f32,
        color: [f32; 3],
    ) -> Vec<RenderableBatch> {
        // This function is used to convert the AstarNode to a TraceSegment
        // It assumes that the node has a direction and a position
        let opaque_color = [color[0], color[1], color[2], 1.0]; // make the color opaque
        let transparent_color = [color[0], color[1], color[2], 0.5]; // make the color transparent
        // if let Some(direction) = &self.direction {

        // } else {

        // }
        match &self.direction {
            AStarNodeDirection::None => {
                let shape_renderable = ShapeRenderable {
                    shape: PrimShape::Circle(CircleShape {
                        position: self.position.to_float(),
                        diameter: width,
                    }),
                    color: opaque_color,
                };
                let shape_clearance_renderable = ShapeRenderable {
                    shape: PrimShape::Circle(CircleShape {
                        position: self.position.to_float(),
                        diameter: width + clearance * 2.0,
                    }),
                    color: transparent_color,
                };
                vec![RenderableBatch(vec![
                    shape_renderable,
                    shape_clearance_renderable,
                ])]
            }
            AStarNodeDirection::Planar(direction) => {
                // If the node has a direction, we can create a TraceSegment
                let trace_segment = TraceSegment {
                    start: self.prev_node.as_ref().unwrap().position,
                    end: self.position,
                    width,
                    clearance,
                    layer: self.layer, // don't care
                };
                let renderables = trace_segment.to_renderables(opaque_color);
                let clearance_renderables =
                    trace_segment.to_clearance_renderables(transparent_color);
                vec![
                    RenderableBatch(renderables),
                    RenderableBatch(clearance_renderables),
                ]
            }
            AStarNodeDirection::Vertical { from_layer } => {
                // draw a via
                let shape_renderable = ShapeRenderable {
                    shape: PrimShape::Circle(CircleShape {
                        position: self.position.to_float(),
                        diameter: via_diameter,
                    }),
                    color: opaque_color,
                };
                let shape_clearance_renderable = ShapeRenderable {
                    shape: PrimShape::Circle(CircleShape {
                        position: self.position.to_float(),
                        diameter: via_diameter + clearance * 2.0,
                    }),
                    color: transparent_color,
                };
                vec![RenderableBatch(vec![
                    shape_renderable,
                    shape_clearance_renderable,
                ])]
            }
        }
    }
}

pub struct AStarResult {
    pub trace_path: TracePath,
}
