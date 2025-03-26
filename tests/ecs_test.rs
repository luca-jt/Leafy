use falling_leaf::ecs::entity_manager::EntityManager;
use falling_leaf::prelude::*;

struct A;
impl Component for A {}
struct B;
impl Component for B {}
struct C;
impl Component for C {}
struct D;
impl Component for D {}

#[test]
fn entity_test() {
    let mut ecs = EntityManager::new();
    let a = ecs.create_entity(components!(A, B));
    let x = ecs.create_entity(components!(A, B));
    ecs.delete_entity(a).unwrap();
    ecs.add_component(x, C).unwrap();
    ecs.add_component(x, D).unwrap();
    assert!(ecs.has_component::<D>(x));
    ecs.remove_component::<D>(x).unwrap();
    assert!(!ecs.has_component::<D>(x));
    assert!(ecs.has_component::<C>(x));
    assert_eq!(unsafe { ecs.query2::<&A, &B>((None, None)) }.count(), 1);
    ecs.delete_entity(x).unwrap();
}

#[test]
fn render_data() {
    let mut ecs = EntityManager::new();
    let _ = ecs.create_entity(components!(Position::origin(), DirectionalLight::default()));
    let _ = ecs.create_entity(components!(Position::origin()));
    let _ = ecs.create_entity(components!(Position::origin()));
    assert_eq!(
        unsafe { ecs.query1::<&DirectionalLight>((None, None)) }.count(),
        1
    );
    assert_eq!(unsafe { ecs.query1::<&Position>((None, None)) }.count(), 3);
}
