//use std::iter::FromIterator;

use legion::world::SubWorld;
use legion::*;

use crate::comp;

pub trait SortRootEntity {
    /// sort root widget ordered by high depth to low depth
    fn sort_from_near(world: &SubWorld) -> Vec<Entity>;

    /// sort root widgets ordered by low depth to high depth
    fn sort_from_far(world: &SubWorld) -> Vec<Entity>;
}

type RootEntities<'a> = (Entity, &'a comp::Root);

impl<'a> SortRootEntity for RootEntities<'a> {
    fn sort_from_near(world: &SubWorld) -> Vec<Entity> {
        let mut roots = <RootEntities>::query().iter(world).collect::<Vec<_>>();
        roots.sort_by(|(_, root_a), (_, root_b)| root_b.partial_cmp(root_a).unwrap());
        roots.iter().map(|(entity, _)| **entity).collect::<Vec<_>>()
    }

    fn sort_from_far(world: &SubWorld) -> Vec<Entity> {
        let mut roots = <RootEntities>::query().iter(world).collect::<Vec<_>>();
        roots.sort_by(|(_, root_a), (_, root_b)| root_a.partial_cmp(root_b).unwrap());
        roots.iter().map(|(entity, _)| **entity).collect::<Vec<_>>()
    }
}
