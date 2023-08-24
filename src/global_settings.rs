
use bevy_rapier3d::geometry::Group;

pub struct NamedCollisionGroup;
impl NamedCollisionGroup {
    pub const ALL: Group = Group::ALL;

    // pub const SOLID: Group = TERRAIN | OBJECT | CHARACTER;   // operator must be const
    pub const TERRAIN: Group = Group::GROUP_1;
    pub const OBJECT: Group = Group::GROUP_2;
    pub const CHARACTER: Group = Group::GROUP_3;
    pub const PROJECTILE: Group = Group::GROUP_4;

    pub const PURE_SENSOR: Group = Group::GROUP_8;
}
