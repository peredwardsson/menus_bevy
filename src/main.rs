use bevy::prelude::*;

const SCREEN_WIDTH: f32 = 1280.;
const SCREEN_HEIGHT: f32 = 896.;

fn main() {
    let plugins = DefaultPlugins;
    plugins.set(
        ImagePlugin::default_nearest()
    ).set(
        WindowPlugin {
            primary_window: Some(
                Window {
                    resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                    ..default()
                }
            ),
            ..default()
        }
    );
    App::new()
        .add_plugins(DefaultPlugins.set(
            ImagePlugin::default_nearest(),
        ))
        .add_plugins((menu::Menu, game::Game, manager::Mgr))
        .add_systems(Startup, setup)
        .insert_resource(Msaa::Off)
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.transform.translation.x = SCREEN_WIDTH/2.;
    camera.transform.translation.y = SCREEN_HEIGHT/2.;
    commands.spawn(camera);
}
fn despawn_recursive<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    to_despawn.for_each(
        |e| commands.entity(e).despawn_recursive()
    );
}

mod manager {
    use bevy::prelude::*;
    use bevy_asset_loader::prelude::*;

    pub struct Mgr;

    #[derive(AssetCollection, Resource)]
    pub struct SpritesheetAssets {
        #[asset(texture_atlas(tile_size_x = 24., tile_size_y = 24., columns = 12, rows = 4))]
        #[asset(path = "main_character_ss.png")]
        pub main_char: Handle<TextureAtlas>,
        #[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128., columns = 13, rows = 8))]
        #[asset(path = "sokoban.png")]
        pub sokoban: Handle<TextureAtlas>,
        #[asset(path = "star.png")]
        pub star: Handle<Image>,
    }

    impl Plugin for Mgr {
        fn build(&self, app: &mut App) {
            app.add_state::<GameState>()
                .add_loading_state(
                    LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu)
                )
                .add_collection_to_loading_state::<_, SpritesheetAssets>(GameState::Loading)
                ;
        }
    }


    #[derive(Component, Default, States, Debug, Hash, PartialEq, Eq, Clone)]
    pub enum GameState {
        #[default]
        Loading,
        Menu,
        Game,
    }
}

mod menu {

    mod colors {
        use bevy::prelude::*;
        pub const MENU_TEXT_COLOR: Color = Color::DARK_GRAY;
        pub const MENU_BG_COLOR: Color = Color::ANTIQUE_WHITE;
    }
    use crate::despawn_recursive;

    use super::manager::GameState;
    use bevy::{app::AppExit, prelude::*};
    pub struct Menu;

    #[derive(Component)]
    pub struct MenuRoot;

    #[derive(Component, Debug)]
    enum Buttons {
        Play,
        Quit,
    }

    impl Plugin for Menu {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Menu), setup)
                .add_systems(Update, button_system.run_if(in_state(GameState::Menu)))
                .add_systems(OnExit(GameState::Menu), despawn_recursive::<MenuRoot>);
        }
    }

    fn setup(mut commands: Commands) {
        let bg = NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: colors::MENU_BG_COLOR.into(),
            ..default()
        };
        let label = TextBundle::from_section(
            "MinimalMaze",
            TextStyle {
                font_size: 80.0,
                color: colors::MENU_TEXT_COLOR.into(),
                ..default()
            },
        );
        let button_bundle = ButtonBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(40.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            color: Color::BEIGE.into(),
            ..default()
        };
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: colors::MENU_BG_COLOR.into(),
                    ..default()
                },
                MenuRoot,
            ))
            .with_children(|base| {
                base.spawn(bg).with_children(|background| {
                    background.spawn(label);
                    background
                        .spawn((button_bundle.clone(), Buttons::Play))
                        .with_children(|button| {
                            button
                                .spawn(TextBundle::from_section("Play", button_text_style.clone()));
                        });
                    background
                        .spawn((button_bundle, Buttons::Quit))
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section("Quit", button_text_style));
                        });
                });
            });
    }
    fn button_system(
        button_query: Query<(&Buttons, &Interaction), (Changed<Interaction>, With<Buttons>)>,
        mut exit_event: EventWriter<AppExit>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (button, interaction) in &button_query {
            if interaction == &Interaction::Pressed {
                match button {
                    Buttons::Play => {
                        // change gamestate to play
                        game_state.set(GameState::Game)
                    }
                    Buttons::Quit => {
                        exit_event.send(AppExit);
                    }
                }
            }
        }
    }
}

mod game {
    use crate::manager::SpritesheetAssets;

    use super::manager::GameState;
    use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::{collide, Collision}}};
    use bevy_ecs_ldtk::prelude::*;

    pub struct Game;
    impl Plugin for Game {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                 (
                    debug_cmds,
                    listen_for_input,
                    animate_sprite,
                    update_player_sprite_facing,
                ).run_if(in_state(GameState::Game))
            )
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (
                    collide_crates,
                    update_player_movement,
                ).chain().run_if(in_state(GameState::Game))
            )
            .add_plugins(LdtkPlugin)
            .add_event::<DebugCmdEvent>()
            .add_event::<ChangePlayerFacing>()
            .insert_resource(PlayerMovement::default())
            .insert_resource(PlayerSpeed(450))
            .insert_resource(PushSpeed(250))
            .register_ldtk_entity::<LdtkBox>("Box")
            .insert_resource(LevelSelection::Index(0))
            ;
        }
    }

    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let levels = asset_server.load("levels/first_try.ldtk");
        commands.spawn(LdtkWorldBundle {
            ldtk_handle: levels,
            // transform: Transform::from_xyz(-1280./2., -720./2., 0.).into(),
            ..Default::default()
        });
    }

    #[derive(Default, Bundle, LdtkEntity)]
    struct LdtkBox {
        a: Box,
        #[sprite_sheet_bundle]
        sprite_bundle: SpriteSheetBundle,
        #[grid_coords]
        grid_coods: GridCoords,
    }

    #[derive(Event)]
    struct PushBoxEvent {
        which_box: Entity,
        direction: Direction,
    }

    // I think this one is useful too??
    #[derive(Resource)]
    struct BoxesBeingPushed(Vec<Entity>);

    struct AnimationIndices {
        north: [usize; 2],
        west: [usize; 2],
        east: [usize; 2],
        south: [usize; 2],
    }

    const ANIMATION_INDICES: AnimationIndices = AnimationIndices {
        south: [53, 54],
        north: [56, 57],
        west: [79, 80],
        east: [82, 83],
    };

    #[derive(Default)]
    enum DebugCommands {
        #[default]
        SpawnPlayer,
        SpawnStar,
        SpawnCrate,
    }

    #[derive(Event, Default, Deref, DerefMut)]
    struct DebugCmdEvent(DebugCommands);

    #[derive(Event, Deref, DerefMut)]
    struct ChangePlayerFacing(Direction);

    #[derive(Component)]
    struct AnimationIdc {
        first: usize,
        last: usize
    }

    #[derive(Component, Deref, DerefMut)]
    struct AnimationTimer(Timer);

    #[derive(Resource, Deref)]
    struct RunSpeed(u8);

    #[derive(Resource, Deref)]
    struct PushSpeed(u8);

    #[derive(Component)]
    struct PlayerPawn;

    #[derive(Default)]
    enum Direction {
        #[default]
        Up,
        Left,
        Down,
        Right
    }

    #[derive(Default, Resource, Deref, DerefMut)]
    struct PlayerMovement {
        #[deref]
        direction: Direction,
        should_move: bool
    }

    #[derive(Resource, Deref)]
    struct PlayerSpeed(u16);

    #[derive(Component, Default)]
    struct Box;

    #[derive(Component)]
    struct Wall;

    #[derive(Component)]
    struct Blocker;

    fn collide_crates(
        mut crates: Query<&mut Transform, With<Box>>,
        mut player: Query<&mut Transform, (With<PlayerPawn>, Without<Box>)>,
        dt: Res<Time>,
        push_speed: Res<PushSpeed>,
        mut player_movement: ResMut<PlayerMovement>,
    ) {
        for mut box_tf in crates.iter_mut() {
            let Ok(player) = player.get_single_mut() else {return;};
            let player_size = Vec2::splat(128.);
            let box_size = Vec2::splat(128.);
            let Some(c) = collide(player.translation, player_size, box_tf.translation, box_size) else {
                continue;
            };
            let speed = (push_speed.0) as f32 * dt.delta_seconds();
            match c {
                Collision::Left => {
                    box_tf.translation.x += speed;
                    player_movement.should_move = false;
                },
                Collision::Right => {
                    box_tf.translation.x -= speed;
                    player_movement.should_move = false;
                },
                Collision::Top => {
                    box_tf.translation.y -= speed;
                    player_movement.should_move = false;
                },
                Collision::Bottom => {
                    box_tf.translation.y += speed;
                    player_movement.should_move = false;
                },
                Collision::Inside => { },
            }
        }
    }

    fn animate_sprite(
        mut animatable_sprites: Query<
            (&AnimationIdc,
            &mut AnimationTimer,
            &mut TextureAtlasSprite)
        >,
        player_movement: ResMut<PlayerMovement>,
        time: Res<Time>
    ) {
        for (idc, mut timer,  mut sprite) in &mut animatable_sprites {
            timer.tick(time.delta());
            if timer.just_finished() && player_movement.should_move {
                sprite.index = if sprite.index >= idc.last {
                    idc.first
                } else {
                    sprite.index + 1
                }
            }
        }
    }

    fn debug_cmds(
        mut commands: Commands,
        mut ev_spawn: EventReader<DebugCmdEvent>,
        assets: Res<SpritesheetAssets>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for _e in ev_spawn.iter() {
            match _e.0 {
                DebugCommands::SpawnPlayer => {

                    let idc = AnimationIdc{
                        first: ANIMATION_INDICES.south[0],
                        last: ANIMATION_INDICES.south[1]
                    };
                    let mut sprite = TextureAtlasSprite::new(idc.first);
                    sprite.custom_size = Some(Vec2::new(24., 24.));
                    let transform = {
                        let mut tf = Transform::from_scale(Vec3::splat(6.0));
                        tf.translation.z = 10.0;
                        tf
                    };
                    info!("Spawning player at {:?}", transform);
                    commands.spawn(
                        (SpriteSheetBundle {
                            // texture_atlas: assets.main_char.clone(),
                            texture_atlas: assets.sokoban.clone(),
                            sprite,
                            transform,
                            ..default()
                        },
                        idc,
                        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
                        PlayerPawn
                    )
                );
            },
            DebugCommands::SpawnStar => {

                commands.spawn(
                    SpriteBundle {
                        texture: assets.star.clone(),
                        transform: Transform::from_scale(Vec3::splat(0.1)),
                        ..default()
                    }
                );
            },
            DebugCommands::SpawnCrate => {

                commands.spawn(
                    (
                        MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Quad::new(Vec2::splat(124.)).into()).into(),
                            material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
                            transform: Transform::from_translation(Vec3::new(200., 0., 0.)),
                            ..default()
                        },
                        Box
                    )
                );
            }
            }
        }
    }

    fn listen_for_input(
        mut game_state: ResMut<NextState<GameState>>,
        inputs: Res<Input<KeyCode>>,
        mut spawn_player_event: EventWriter<DebugCmdEvent>,
        mut player_movement: ResMut<PlayerMovement>,
        mut player_facing_evw: EventWriter<ChangePlayerFacing>,
    ) {

        if inputs.just_pressed(KeyCode::Escape) {
            game_state.set(GameState::Menu);
        }
        if inputs.just_pressed(KeyCode::Key0) {
            spawn_player_event.send(DebugCmdEvent(DebugCommands::SpawnPlayer))
        }
        if inputs.just_pressed(KeyCode::O) {
            spawn_player_event.send(DebugCmdEvent(DebugCommands::SpawnStar))
        }
        if inputs.just_pressed(KeyCode::P) {
            spawn_player_event.send(DebugCmdEvent(DebugCommands::SpawnCrate))
        }

        if inputs.just_pressed(KeyCode::W) {
            // move up
            **player_movement = Direction::Up;
            player_facing_evw.send(ChangePlayerFacing(Direction::Up));
        }

        if inputs.just_pressed(KeyCode::A) {
            // move left
            **player_movement = Direction::Left;
            player_facing_evw.send(ChangePlayerFacing(Direction::Left))
        }

        if inputs.just_pressed(KeyCode::S) {
            // move down
            **player_movement = Direction::Down;
            player_facing_evw.send(ChangePlayerFacing(Direction::Down))
        }

        if inputs.just_pressed(KeyCode::D) {
            // move right
            **player_movement = Direction::Right;
            player_facing_evw.send(ChangePlayerFacing(Direction::Right))
        }

        if inputs.any_pressed([KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D]) {
            player_movement.should_move = true;
        } else {
            player_movement.should_move = false;
        }
    }

    fn update_player_movement(
        mut players: Query<&mut Transform, With<PlayerPawn>>,
        movement: Res<PlayerMovement>,
        speed_res: Res<PlayerSpeed>,
        time: Res<Time>,
    ) {
        let Ok(mut player) = players.get_single_mut() else {return;};
        let speed = **speed_res as f32 * time.delta_seconds();
        if movement.should_move {
            match movement.direction {
                Direction::Down => player.translation.y -= speed,
                Direction::Left => player.translation.x -= speed,
                Direction::Right => player.translation.x += speed,
                Direction::Up => player.translation.y += speed,
            };
        }
    }

    fn update_player_sprite_facing(
        mut players: Query<(&mut AnimationIdc, &mut TextureAtlasSprite), With<PlayerPawn>>,
        // mut player_movement: ResMut<PlayerMovement>,
        mut player_facing_evr: EventReader<ChangePlayerFacing>,
    ) {
        for evt in player_facing_evr.iter() {
            let Ok((mut idc, mut sprite)) = players.get_single_mut() else {return;};
            match **evt {
                Direction::Down => {
                    idc.first = ANIMATION_INDICES.south[0];
                    idc.last = ANIMATION_INDICES.south[1];
                    sprite.index = idc.first
                },
                Direction::Up => {
                    idc.first = ANIMATION_INDICES.north[0];
                    idc.last = ANIMATION_INDICES.north[1];
                    sprite.index = idc.first
                },
                Direction::Right => {
                    idc.first = ANIMATION_INDICES.west[0];
                    idc.last = ANIMATION_INDICES.west[1];
                    sprite.index = idc.first
                },
                Direction::Left => {
                    idc.first = ANIMATION_INDICES.east[0];
                    idc.last = ANIMATION_INDICES.east[1];
                    sprite.index = idc.first
                },
            }
        }
    }
}
