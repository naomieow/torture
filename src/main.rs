use valence::{
    prelude::*, 
    DefaultPlugins, 
    inventory::HeldItem, 
    client::hand_swing::HandSwingEvent, 
    entity::{
        zombie::ZombieEntityBundle, 
        entity::NameVisible
    }, 
    network::{
        async_trait, 
        BroadcastToLan
    }
};

const SPAWN_Y: i32 = 64;

struct TortureCallbacks;

#[async_trait]
impl NetworkCallbacks for TortureCallbacks {
    async fn broadcast_to_lan(&self, _shared: &SharedNetworkState) -> BroadcastToLan {
        BroadcastToLan::Enabled("Connect!".into())
    }
}

fn main() {
    let mut app = App::new();
    build_app(&mut app);
    app.run();
}

fn build_app(app: &mut App) {
    app.insert_resource(NetworkSettings {
        connection_mode: ConnectionMode::Online { prevent_proxy_connections: false },
        callbacks: TortureCallbacks.into(),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            init_clients,
            despawn_disconnected_clients,
            on_client_click
        ),
    )
    .run();
}

fn on_client_click(
    mut clients: Query<&HeldItem>,
    mut _instances: Query<&mut Instance>,
    mut hand_swing_events: EventReader<HandSwingEvent>,
) {
    let _instance = _instances.single();

    for event in hand_swing_events.iter() {
        let Ok(held) = clients.get_mut(event.client) else {
            continue;
        };

        let bar_slot = held.slot() - 36;
    
        if bar_slot == 0 {
            println!("Slot 0 swung");
        }
    }
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    mut dimensions: ResMut<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    dimensions.insert(
        ident!("overworld"),
        DimensionType {
            ambient_light: 1.0,
            has_skylight: false,
            has_ceiling: false,
            natural: false,
            ..Default::default()
        },
    );

    let mut instance = Instance::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            instance.insert_chunk([x, z], Chunk::default());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            instance.set_block([x, SPAWN_Y, z], BlockState::GRASS_BLOCK);
        }
    }

    for y in 0..5 {
        for x in -5..5 {
            instance.set_block([x, SPAWN_Y + y, 5], BlockState::STONE_BRICKS);
        }
    }

    instance.set_block(
        [0, SPAWN_Y+2, 4], 
        BlockState::WALL_TORCH.set(PropName::Facing, PropValue::North),
    );

    let instance_id = commands.spawn(instance).id();

    commands.spawn(ZombieEntityBundle {
        location: Location(instance_id),
        position: Position(DVec3::new(4.0, SPAWN_Y as f64 + 1.0, 1.0)),
        look: Look::new(180.0, 0.0),
        head_yaw: HeadYaw(135.0),
        entity_name_visible: NameVisible(true),
        ..Default::default()
    });

    commands.spawn(ZombieEntityBundle {
        location: Location(instance_id),
        position: Position(DVec3::new(-4.0, SPAWN_Y as f64 + 1.0, 1.0)),
        look: Look::new(180.0, 0.0),
        head_yaw: HeadYaw(135.0),
        entity_name_visible: NameVisible(true),
        ..Default::default()
    });
}

fn init_clients(
    mut clients: Query<
        (
            &mut Location,
            &mut Position,
            &mut HasRespawnScreen,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    instances: Query<Entity, With<Instance>>,
) {
    for (mut loc, mut pos, mut has_repsawn_screen, mut game_mode) in &mut clients {
        loc.0 = instances.iter().next().unwrap();
        pos.set([0.0, SPAWN_Y as f64 + 1.0, 0.0]);
        has_repsawn_screen.0 = true;
        *game_mode = GameMode::Adventure;
    }
}
