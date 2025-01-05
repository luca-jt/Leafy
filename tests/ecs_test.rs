use falling_leaf::components;
use falling_leaf::ecs::component::utils::Color32;
use falling_leaf::ecs::component::{MeshAttribute, MeshType, PointLight, Position};
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
    assert_eq!(ecs.query2::<A, B>(None).count(), 1);
    ecs.delete_entity(x).unwrap();
}

#[test]
fn render_data() {
    let mut ecs = EntityManager::new();
    ecs.create_point_light(Position::origin());
    let l = ecs.create_point_light_visible(Position::origin());
    let r1 = ecs.create_entity(components!(
        Position::origin(),
        MeshType::Cone,
        MeshAttribute::Colored(Color32::WHITE)
    ));
    let r2 = ecs.create_entity(components!(
        Position::origin(),
        MeshType::Cube,
        MeshAttribute::Colored(Color32::RED)
    ));
    assert_eq!(ecs.query1::<PointLight>(None).count(), 2);
    assert_eq!(ecs.query1::<MeshType>(None).count(), 3);
    assert_eq!(
        ecs.get_component::<MeshAttribute>(r1)
            .unwrap()
            .color()
            .unwrap(),
        Color32::WHITE
    );
    assert_eq!(
        ecs.get_component::<MeshAttribute>(r2)
            .unwrap()
            .color()
            .unwrap(),
        Color32::RED
    );
    assert_eq!(
        ecs.get_component::<MeshAttribute>(l)
            .unwrap()
            .color()
            .unwrap(),
        Color32::from_rgb(255, 255, 200)
    );
}
