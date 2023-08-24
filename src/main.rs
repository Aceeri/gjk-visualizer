use bevy::prelude::*;
use parry3d::math::{Point, Real, Vector};
use parry3d::na::Unit;
use parry3d::shape::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct VisualizeShape(SharedShape);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_systems(
        Startup,
        (setup_shape, apply_deferred, visualize_shape)
            .chain()
            .into_configs(),
    );

    app.run()
}

fn setup_shape(mut commands: Commands) {
    commands.spawn(VisualizeShape(SharedShape::ball(0.5)));
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle::default());
}

pub struct TaperedCapsule {
    radius_a: f32,
    radius_b: f32,
    segment: Segment,
}

impl SupportMap for TaperedCapsule {
    fn local_support_point(&self, dir: &Vector<f32>) -> Point<f32> {
        let dir = Unit::try_new(*dir, 0.0).unwrap_or(Vector::y_axis());
        self.local_support_point_toward(&dir)
    }

    fn local_support_point_toward(&self, dir: &Unit<Vector<Real>>) -> Point<f32> {
        if dir.dot(&self.segment.a.coords) > dir.dot(&self.segment.b.coords) {
            self.segment.a + **dir * self.radius_a
        } else {
            self.segment.b + **dir * self.radius_b
        }
    }
}

fn visualize_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(bevy::render::mesh::shape::UVSphere {
        radius: 0.01,
        ..default()
    }));
    let material = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        perceptual_roughness: 0.5,
        ..default()
    });

    let samples = 100_000;

    // golden angle in radians
    let phi = PI * (5.0f32).sqrt() - 1.0;

    //let support_map = Cuboid::new(Vector::new(0.5, 0.5, 0.5));
    //let support_map = Ball::new(0.5);
    /*
    let support_map = Capsule {
        segment: Segment {
            a: Vector::new(0.0, 0.5, 0.0).into(),
            b: Vector::new(0.0, -0.5, 0.0).into(),
        },
        radius: 0.5,
    };
    */
    //let support_map = Cone::new(0.5, 0.5);
    let support_map = TaperedCapsule {
        segment: Segment {
            a: Vector::new(0.0, 0.5, 0.0).into(),
            b: Vector::new(0.0, -0.5, 0.0).into(),
        },
        radius_a: 0.5,
        radius_b: 0.2,
    };

    for i in 0..samples {
        let i = i as f32;
        let y = 1.0 - (i / (samples - 1) as f32) * 2.0;
        let radius = (1.0 - y * y).sqrt();
        let theta = phi * i;

        let x = theta.cos() * radius;
        let z = theta.sin() * radius;

        let sample_dir = Vec3::new(x, y, z);

        let sample_dir =
            parry3d::math::Vector::<f32>::new(sample_dir.x, sample_dir.y, sample_dir.z);
        let point = SupportMap::local_support_point(&support_map, &sample_dir);
        //let point = support_map.local_support_point_towards(sample_dir);
        let point = Vec3::new(point.x, point.y, point.z);

        commands.spawn(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform {
                translation: point,
                ..default()
            },
            ..default()
        });
    }
}
