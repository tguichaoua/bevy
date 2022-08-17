use bevy_ecs::{
    prelude::Entity,
    query::{With, Without},
    system::Query,
};

use crate::{Children, ChildrenQuery, LocalQuery, Parent};

use super::{Propagate, PropagateKind};

pub struct AlwaysPropagate;

impl PropagateKind for AlwaysPropagate {
    type RootQuery<'w, 's, T: Propagate<Kind = Self>> = Query<
        'w,
        's,
        (
            Option<&'static Children>,
            &'static T,
            &'static mut T::Computed,
            Entity,
        ),
        Without<Parent>,
    >;

    type LocalQuery<'w, 's, T: Propagate<Kind = Self>> =
        Query<'w, 's, (&'static T, &'static mut T::Computed, &'static Parent)>;

    type ChildrenQuery<'w, 's, T: Propagate<Kind = Self>> =
        Query<'w, 's, &'static Children, (With<Parent>, With<T>, With<T::Computed>)>;

    fn propagate<T: Propagate<Kind = Self>>(
        mut root_query: Self::RootQuery<'_, '_, T>,
        mut local_query: Self::LocalQuery<'_, '_, T>,
        children_query: Self::ChildrenQuery<'_, '_, T>,
    ) {
        for (children, local, mut computed, entity) in root_query.iter_mut() {
            T::compute_root(computed.as_mut(), local);

            if let Some(children) = children {
                let payload = T::payload(computed.as_ref());
                for child in children.iter() {
                    let _ = propagate_recursive::<T>(
                        &payload,
                        &mut local_query,
                        &children_query,
                        *child,
                        entity,
                    );
                }
            }
        }
    }
}

fn propagate_recursive<T: Propagate<Kind = AlwaysPropagate>>(
    payload: &T::Payload,
    local_query: &mut LocalQuery<T>,
    children_query: &ChildrenQuery<T>,
    entity: Entity,
    expected_parent: Entity,
    // BLOCKED: https://github.com/rust-lang/rust/issues/31436
    // We use a result here to use the `?` operator. Ideally we'd use a try block instead
) -> Result<(), ()> {
    let payload = {
        let (local, mut computed, child_parent) = local_query.get_mut(entity).map_err(drop)?;
        assert_eq!(
            child_parent.get(), expected_parent,
            "Malformed hierarchy. This probably means that your hierarchy has been improperly maintained, or contains a cycle"
        );
        T::compute(computed.as_mut(), payload, local);
        T::payload(computed.as_ref())
    };

    for child in children_query.get(entity).map_err(drop)?.iter() {
        let _ = propagate_recursive::<T>(&payload, local_query, children_query, *child, entity);
    }
    Ok(())
}
