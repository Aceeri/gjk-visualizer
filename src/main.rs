use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::*;
use parry3d::math::{Point, Real, Vector};
use parry3d::na::Unit;
use parry3d::shape::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct VisualizeShape(SharedShape);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PanOrbitCameraPlugin);

    app.add_systems(
        Startup,
        (setup_shape, apply_deferred, visualize_shape)
            .chain()
            .into_configs(),
    );

    app.run()
}

#[derive(Resource, Default)]
pub struct CameraControls {
    pub pitch: f32,
    pub yaw: f32,
}

fn setup_shape(mut commands: Commands) {
    commands.spawn(VisualizeShape(SharedShape::ball(0.5)));
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PanOrbitCamera::default());

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::ORANGE_RED,
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(-1.0, -1.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::MIDNIGHT_BLUE,
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::ANTIQUE_WHITE,
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::ANTIQUE_WHITE,
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, -1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
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
    let grey = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        perceptual_roughness: 0.5,
        ..default()
    });
    let white = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        ..default()
    });

    let samples = 500;

    // golden angle in radians
    let phi = PI * (5.0f32).sqrt() - 1.0;

    let support_map = Cuboid::new(Vector::new(0.5, 0.5, 0.5));

    //let support_map = Ball::new(0.5);
    /*let support_map = Capsule {
        segment: Segment {
            a: Vector::new(0.0, 0.5, 0.0).into(),
            b: Vector::new(0.0, -0.5, 0.0).into(),
        },
        radius: 0.5,
    };*/
    //let support_map = Cone::new(0.5, 0.5);
    /*
    let support_map = TaperedCapsule {
        segment: Segment {
            a: Vector::new(0.0, 0.5, 0.0).into(),
            b: Vector::new(0.0, -0.5, 0.0).into(),
        },
        radius_a: 0.5,
        radius_b: 0.2,
    };
    */

    let mut points = Vec::new();
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
        points.push(point);
        //let point = support_map.local_support_point_towards(sample_dir);
        let point = Vec3::new(point.x, point.y, point.z);
        /*
        commands.spawn(PbrBundle {
            mesh: mesh.clone(),
            material: white.clone(),
            transform: Transform {
                translation: point,
                ..default()
            },
            ..default()
        });
        */
    }

    let convex_hull = ConvexPolyhedron::from_convex_hull(points.as_slice()).unwrap();
    let mesh = to_mesh(convex_hull);
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh.clone()),
        material: grey.clone(),
        transform: Transform::default(),
        ..default()
    });
}

pub fn trimesh_to_mesh(trimesh: &TriMesh) -> Mesh {
    let points = trimesh.vertices();
    let indices = trimesh.indices();
    let points: Vec<[f32; 3]> = points
        .iter()
        .map(|point| [point.x, point.y, point.z])
        .collect();
    let indices: Vec<u32> = indices.iter().flatten().cloned().collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}

fn to_mesh(convex_polyhedron: ConvexPolyhedron) -> Mesh {
    let (points, indices) = convex_polyhedron.to_trimesh();
    let trimesh = TriMesh::new(points, indices);
    let mesh = trimesh_to_mesh(&trimesh);
    mesh
}
