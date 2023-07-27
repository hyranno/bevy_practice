use std::ops::DerefMut;

use bevy::prelude::*;

use super::button_like::ButtonLike;


pub fn update_four_button_axis <Axis, NegativeX, PositiveX, NegativeY, PositiveY> (
    mut query: Query<
        (
            &mut Axis,
            &NegativeX, &PositiveX,
            &NegativeY, &PositiveY,
        ),
        Or<(Changed<NegativeX>, Changed<PositiveX>, Changed<NegativeY>, Changed<PositiveY>)>
    >,
) where
    Axis: Component + DerefMut<Target = Vec2>,
    NegativeX: Component + ButtonLike,
    PositiveX: Component + ButtonLike,
    NegativeY: Component + ButtonLike,
    PositiveY: Component + ButtonLike,
{
    for (mut axis, negative_x, positive_x, negative_y, positive_y) in query.iter_mut() {
        let value = Vec2::new(
            if negative_x.is_pressed() {-1.0} else {0.0} + if positive_x.is_pressed() {1.0} else {0.0},
            if negative_y.is_pressed() {-1.0} else {0.0} + if positive_y.is_pressed() {1.0} else {0.0},
        );
        // check real change for component change detection
        if **axis != value {
            **axis = value;
        }
    }
}

