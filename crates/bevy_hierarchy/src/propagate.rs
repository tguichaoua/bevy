#![allow(missing_docs)]

mod always;
mod if_changed;

pub use always::*;
pub use if_changed::*;

use bevy_ecs::{prelude::*, system::SystemParam};

pub trait PropagateKind {
    type RootQuery<'w, 's, T: Propagate<Kind = Self>>: SystemParam;
    type LocalQuery<'w, 's, T: Propagate<Kind = Self>>: SystemParam;
    type ChildrenQuery<'w, 's, T: Propagate<Kind = Self>>: SystemParam;

    fn propagate<T: Propagate<Kind = Self>>(
        root_query: Self::RootQuery<'_, '_, T>,
        local_query: Self::LocalQuery<'_, '_, T>,
        children_query: Self::ChildrenQuery<'_, '_, T>,
    );
}

/// Marks a component as propagatable thrown hierachy alike `Transform`/`GlobalTransorm`
/// or `Visibility`/`ComputedVisibility`.
pub trait Propagate: Component {
    /// The computed version of this component.
    type Computed: Component;
    /// The payload passed to children for computation.
    type Payload;

    type Kind: PropagateKind;

    /// Update computed component for root entity from it's local component.
    fn compute_root(computed: &mut Self::Computed, local: &Self);

    /// Update computed component from the parent's payload and the local component.
    fn compute(computed: &mut Self::Computed, payload: &Self::Payload, local: &Self);

    /// Compute the payload to pass to children from the computed component.
    fn payload(computed: &Self::Computed) -> Self::Payload;
}

pub type RootQuery<'w, 's, T> = <<T as Propagate>::Kind as PropagateKind>::RootQuery<'w, 's, T>;

pub type LocalQuery<'w, 's, T> = <<T as Propagate>::Kind as PropagateKind>::LocalQuery<'w, 's, T>;

pub type ChildrenQuery<'w, 's, T> =
    <<T as Propagate>::Kind as PropagateKind>::ChildrenQuery<'w, 's, T>;

/// Update `T::Computed` component of entities based on entity hierarchy and
/// `T` component.
pub fn propagate_system<T: Propagate>(
    root_query: RootQuery<T>,
    local_query: LocalQuery<T>,
    children_query: ChildrenQuery<T>,
) {
    T::Kind::propagate(root_query, local_query, children_query);
}

// #[cfg(test)]
// mod test {
//     use bevy_app::App;
//     use bevy_ecs::prelude::*;
//     use bevy_ecs::system::CommandQueue;

//     use crate::{propagate_system, BuildChildren, BuildWorldChildren, Children, Parent, Propagate};

//     #[derive(Component)]
//     struct MyComponent(i32);

//     #[derive(Default, Component, Clone, Copy)]
//     struct MyComputedComponent(i32);

//     impl MyComponent {
//         const IDENTITY: Self = Self(1);
//     }

//     impl Default for MyComponent {
//         fn default() -> Self {
//             Self::IDENTITY
//         }
//     }

//     impl Propagate for MyComponent {
//         type Computed = MyComputedComponent;
//         type Payload = MyComputedComponent;
//         const ALWAYS_PROPAGATE: bool = false;

//         fn compute_root(computed: &mut Self::Computed, local: &Self) {
//             computed.0 = local.0;
//         }

//         fn compute(computed: &mut Self::Computed, payload: &Self::Payload, local: &Self) {
//             computed.0 = payload.0 * local.0;
//         }

//         fn payload(computed: &Self::Computed) -> Self::Payload {
//             *computed
//         }
//     }

//     #[test]
//     fn did_propagate() {
//         let mut world = World::default();

//         let mut update_stage = SystemStage::parallel();
//         update_stage.add_system(propagate_system::<MyComponent>);

//         let mut schedule = Schedule::default();
//         schedule.add_stage("update", update_stage);

//         const ROOT_VALUE: i32 = 5;
//         const CHILDREN_0_VALUE: i32 = 3;
//         const CHILDREN_1_VALUE: i32 = -2;

//         let mut children = Vec::new();
//         world
//             .spawn()
//             .insert_bundle((MyComponent(ROOT_VALUE), MyComputedComponent::default()))
//             .with_children(|parent| {
//                 children.push(
//                     parent
//                         .spawn_bundle((
//                             MyComponent(CHILDREN_0_VALUE),
//                             MyComputedComponent::default(),
//                         ))
//                         .id(),
//                 );
//                 children.push(
//                     parent
//                         .spawn_bundle((
//                             MyComponent(CHILDREN_1_VALUE),
//                             MyComputedComponent::default(),
//                         ))
//                         .id(),
//                 );
//             });
//         schedule.run(&mut world);

//         assert_eq!(
//             world.get::<MyComputedComponent>(children[0]).unwrap().0,
//             ROOT_VALUE * CHILDREN_0_VALUE
//         );

//         assert_eq!(
//             world.get::<MyComputedComponent>(children[1]).unwrap().0,
//             ROOT_VALUE * CHILDREN_1_VALUE
//         );
//     }

//     #[test]
//     fn did_propagate_command_buffer() {
//         let mut world = World::default();
//         let mut update_stage = SystemStage::parallel();
//         update_stage.add_system(propagate_system::<MyComponent>);

//         let mut schedule = Schedule::default();
//         schedule.add_stage("update", update_stage);

//         const ROOT_VALUE: i32 = 5;
//         const CHILDREN_0_VALUE: i32 = 3;
//         const CHILDREN_1_VALUE: i32 = -2;

//         // Root entity
//         let mut queue = CommandQueue::default();
//         let mut commands = Commands::new(&mut queue, &world);
//         let mut children = Vec::new();
//         commands
//             .spawn_bundle((MyComponent(ROOT_VALUE), MyComputedComponent::default()))
//             .with_children(|parent| {
//                 children.push(
//                     parent
//                         .spawn_bundle((
//                             MyComponent(CHILDREN_0_VALUE),
//                             MyComputedComponent::default(),
//                         ))
//                         .id(),
//                 );
//                 children.push(
//                     parent
//                         .spawn_bundle((
//                             MyComponent(CHILDREN_1_VALUE),
//                             MyComputedComponent::default(),
//                         ))
//                         .id(),
//                 );
//             });
//         queue.apply(&mut world);
//         schedule.run(&mut world);

//         assert_eq!(
//             world.get::<MyComputedComponent>(children[0]).unwrap().0,
//             ROOT_VALUE * CHILDREN_0_VALUE
//         );

//         assert_eq!(
//             world.get::<MyComputedComponent>(children[1]).unwrap().0,
//             ROOT_VALUE * CHILDREN_1_VALUE
//         );
//     }

//     #[test]
//     fn correct_children() {
//         let mut world = World::default();

//         let mut update_stage = SystemStage::parallel();
//         update_stage.add_system(propagate_system::<MyComponent>);

//         let mut schedule = Schedule::default();
//         schedule.add_stage("update", update_stage);

//         // Add parent entities
//         let mut children = Vec::new();
//         let parent = {
//             let mut command_queue = CommandQueue::default();
//             let mut commands = Commands::new(&mut command_queue, &world);
//             let parent = commands.spawn().insert(MyComponent::default()).id();
//             commands.entity(parent).with_children(|parent| {
//                 children.push(parent.spawn().insert(MyComponent::default()).id());
//                 children.push(parent.spawn().insert(MyComponent::default()).id());
//             });
//             command_queue.apply(&mut world);
//             schedule.run(&mut world);
//             parent
//         };

//         assert_eq!(
//             world
//                 .get::<Children>(parent)
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<_>>(),
//             children,
//         );

//         // Parent `e1` to `e2`.
//         {
//             let mut command_queue = CommandQueue::default();
//             let mut commands = Commands::new(&mut command_queue, &world);
//             commands.entity(children[1]).add_child(children[0]);
//             command_queue.apply(&mut world);
//             schedule.run(&mut world);
//         }

//         assert_eq!(
//             world
//                 .get::<Children>(parent)
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<_>>(),
//             vec![children[1]]
//         );

//         assert_eq!(
//             world
//                 .get::<Children>(children[1])
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<_>>(),
//             vec![children[0]]
//         );

//         assert!(world.despawn(children[0]));

//         schedule.run(&mut world);

//         assert_eq!(
//             world
//                 .get::<Children>(parent)
//                 .unwrap()
//                 .iter()
//                 .cloned()
//                 .collect::<Vec<_>>(),
//             vec![children[1]]
//         );
//     }

//     #[test]
//     fn correct_when_no_children() {
//         let mut app = App::new();

//         app.add_system(propagate_system::<MyComponent>);

//         const ROOT_VALUE: i32 = 5;

//         // These will be overwritten.
//         let mut child = Entity::from_raw(0);
//         let mut grandchild = Entity::from_raw(1);
//         let parent = app
//             .world
//             .spawn()
//             .insert(MyComponent(ROOT_VALUE))
//             .insert(MyComputedComponent::default())
//             .with_children(|builder| {
//                 child = builder
//                     .spawn_bundle((MyComponent::IDENTITY, MyComputedComponent::default()))
//                     .with_children(|builder| {
//                         grandchild = builder
//                             .spawn_bundle((MyComponent::IDENTITY, MyComputedComponent::default()))
//                             .id();
//                     })
//                     .id();
//             })
//             .id();

//         app.update();

//         // check the `Children` structure is spawned
//         assert_eq!(&**app.world.get::<Children>(parent).unwrap(), &[child]);
//         assert_eq!(&**app.world.get::<Children>(child).unwrap(), &[grandchild]);
//         // Note that at this point, the `GlobalTransform`s will not have updated yet, due to `Commands` delay
//         app.update();

//         let mut state = app.world.query::<&MyComputedComponent>();
//         for global in state.iter(&app.world) {
//             assert_eq!(global.0, ROOT_VALUE);
//         }
//     }

//     #[test]
//     #[should_panic]
//     fn panic_when_hierarchy_cycle() {
//         // We cannot directly edit Parent and Children, so we use a temp world to break
//         // the hierarchy's invariants.
//         let mut temp = World::new();
//         let mut app = App::new();

//         app.add_system(propagate_system::<MyComponent>);

//         fn setup_world(world: &mut World) -> (Entity, Entity) {
//             let mut grandchild = Entity::from_raw(0);
//             let child = world
//                 .spawn()
//                 .insert_bundle((MyComponent::default(), MyComputedComponent::default()))
//                 .with_children(|builder| {
//                     grandchild = builder
//                         .spawn()
//                         .insert_bundle((MyComponent::default(), MyComputedComponent::default()))
//                         .id();
//                 })
//                 .id();
//             (child, grandchild)
//         }

//         let (temp_child, temp_grandchild) = setup_world(&mut temp);
//         let (child, grandchild) = setup_world(&mut app.world);

//         assert_eq!(temp_child, child);
//         assert_eq!(temp_grandchild, grandchild);

//         app.world
//             .spawn()
//             .insert_bundle((MyComponent::default(), MyComputedComponent::default()))
//             .push_children(&[child]);
//         std::mem::swap(
//             &mut *app.world.get_mut::<Parent>(child).unwrap(),
//             &mut *temp.get_mut::<Parent>(grandchild).unwrap(),
//         );

//         app.update();
//     }
// }
