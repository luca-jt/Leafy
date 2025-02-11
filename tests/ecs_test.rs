use falling_leaf::components;
use falling_leaf::ecs::component::*;
use falling_leaf::ecs::entity_manager::EntityManager;

struct A;
struct B;
struct C;
struct D;

#[test]
fn entity_test() {
    let mut ecs = EntityManager::new();
    let x = ecs.create_entity(components!(A, B));
    ecs.add_component(x, C).unwrap();
    ecs.add_component(x, D).unwrap();
    assert!(ecs.has_component::<D>(x));
    ecs.remove_component::<D>(x).unwrap();
    assert!(!ecs.has_component::<D>(x));
    assert!(ecs.has_component::<C>(x));
    assert_eq!(ecs.query2::<A, B>((None, None)).count(), 1);
    ecs.delete_entity(x).unwrap();
}

#[test]
fn render_data() {
    let mut ecs = EntityManager::new();
    let _ = ecs.create_entity(components!(Position::origin(), PointLight::default()));
    let _ = ecs.create_entity(components!(Position::origin()));
    let _ = ecs.create_entity(components!(Position::origin()));
    assert_eq!(ecs.query1::<PointLight>((None, None)).count(), 1);
    assert_eq!(ecs.query1::<Position>((None, None)).count(), 3);
}
