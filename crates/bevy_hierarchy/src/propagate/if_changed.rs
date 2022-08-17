use bevy_ecs::{
    prelude::Entity,
    query::{Changed, With, Without},
    system::Query,
};

use crate::{Children, Parent};

use super::{ChildrenQuery, LocalQuery, Propagate, PropagateKind};

pub struct PropagateIfChanged;

impl PropagateKind for PropagateIfChanged {
    type RootQuery<'w, 's, T: Propagate<Kind = Self>> = Query<
        'w,
        's,
        (
            Option<(&'static Children, Changed<Children>)>,
            &'static T,
            Changed<T>,
            &'static mut T::Computed,
            Entity,
        ),
        Without<Parent>,
    >;

    type LocalQuery<'w, 's, T: Propagate<Kind = Self>> = Query<
        'w,
        's,
        (
            &'static T,
            Changed<T>,
            &'static mut <T as Propagate>::Computed,
            &'static Parent,
        ),
    >;

    type ChildrenQuery<'w, 's, T: Propagate<Kind = Self>> = Query<
        'w,
        's,
        (&'static Children, Changed<Children>),
        (With<Parent>, With<<T as Propagate>::Computed>),
    >;

    fn propagate<T: Propagate<Kind = Self>>(
        mut root_query: Self::RootQuery<'_, '_, T>,
        mut local_query: Self::LocalQuery<'_, '_, T>,
        children_query: Self::ChildrenQuery<'_, '_, T>,
    ) {
        for (children, local, local_changed, mut computed, entity) in root_query.iter_mut() {
            let mut changed = local_changed;
            if changed {
                T::compute_root(computed.as_mut(), local);
            }

            if let Some((children, changed_children)) = children {
                // If our `Children` has changed, we need to recalculate everything below us
                changed |= changed_children;
                let payload = T::payload(computed.as_ref());
                for child in children {
                    let _ = propagate_recursive::<T>(
                        &payload,
                        &mut local_query,
                        &children_query,
                        *child,
                        entity,
                        changed,
                    );
                }
            }
        }
    }
}

fn propagate_recursive<T: Propagate<Kind = PropagateIfChanged>>(
    payload: &T::Payload,
    local_query: &mut LocalQuery<T>,
    children_query: &ChildrenQuery<T>,
    entity: Entity,
    expected_parent: Entity,
    mut changed: bool,
    // BLOCKED: https://github.com/rust-lang/rust/issues/31436
    // We use a result here to use the `?` operator. Ideally we'd use a try block instead
) -> Result<(), ()> {
    let payload = {
        let (local, local_changed, mut computed, child_parent) =
            local_query.get_mut(entity).map_err(drop)?;
        // Note that for parallelising, this check cannot occur here, since there is an `&mut GlobalTransform` (in global_transform)
        assert_eq!(
            child_parent.get(), expected_parent,
            "Malformed hierarchy. This probably means that your hierarchy has been improperly maintained, or contains a cycle"
        );
        changed |= local_changed;
        if changed {
            T::compute(computed.as_mut(), payload, local);
        }
        T::payload(computed.as_ref())
    };

    let (children, changed_children) = children_query.get(entity).map_err(drop)?;
    // If our `Children` has changed, we need to recalculate everything below us
    changed |= changed_children;
    for child in children {
        let _ = propagate_recursive::<T>(
            &payload,
            local_query,
            children_query,
            *child,
            entity,
            changed,
        );
    }
    Ok(())
}
