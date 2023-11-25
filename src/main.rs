use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            ImagePlugin::default_nearest(),
        ))
        .add_plugins((menu::Menu, game::Game, manager::Mgr))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn despawn_recursive<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    to_despawn.for_each(
        |e| commands.entity(e).despawn_recursive()
    );
}

mod manager {
    use bevy::prelude::*;

    pub struct Mgr;

    #[derive(Component)]
    pub struct Manager;

    #[derive(Resource, Default)]
    struct AssetLoading(pub Vec<HandleUntyped>);

    impl Plugin for Mgr {
        fn build(&self, app: &mut App) {
            app.add_state::<GameState>()
                .insert_resource(AssetLoading::default())
                .add_systems(OnEnter(GameState::Loading), load_assets)
                .add_systems(
                    Update,
                    check_if_assets_loaded.run_if(in_state(GameState::Loading)),
                );
        }
    }
    #[derive(Resource, Deref, DerefMut)]
    struct FauxLoadingTimer(Timer);


    fn load_assets(
        // mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut loading: ResMut<AssetLoading>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        let texture_handle: Handle<Image> = asset_server.load("main_character_ss.png");
        loading.0.push(texture_handle.clone_untyped());

        // maybe should be moved elsewhere
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle, Vec2::new(16.0, 16.0),
             12, 4, None, None
        );
        texture_atlases.add(texture_atlas);
    }

    fn check_if_assets_loaded(
        // mut commands: Commands,
        // time: Res<Time>,
        // mut timer: ResMut<FauxLoadingTimer>,
        mut gamestate: ResMut<NextState<GameState>>,
        loading: Res<AssetLoading>,
        server: Res<AssetServer>,
    ) {
        use bevy::asset::LoadState;

        match server.get_group_load_state(loading.0.iter().map(|h| h.id())) {
            LoadState::Failed => {
                // something failed
                println!("Something did not load!! :(");
            },
            LoadState::Loaded => {
                gamestate.set(GameState::Menu);
            },
            _ => {}
        }
        // if timer.tick(time.delta()).finished() {
        //     gamestate.set(GameState::Menu);
        // }
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
    use super::manager::GameState;
    use bevy::prelude::*;
    pub struct Game;
    impl Plugin for Game {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                 (listen_for_input.run_if(in_state(GameState::Game)),
                debug_cmds.run_if(in_state(GameState::Game)),
                animate_sprite,
                update_player_movement,
                update_player_sprite_facing,
            ))
            .add_event::<DebugCmdEvent>()
            .add_event::<ChangePlayerFacing>()
            .insert_resource(PlayerMovement::default())
            .insert_resource(PlayerSpeed(600.0))
            ;
        }
    }

    #[derive(Default)]
    enum DebugCommands {
        #[default]
        SpawnPlayer,
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
    struct Runspeed(u8);

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
    struct PlayerSpeed(f32);

    fn animate_sprite(
        mut animatable_sprites: Query<
            (&AnimationIdc,
            &mut AnimationTimer,
            &mut TextureAtlasSprite)
        >,
        time: Res<Time>
    ) {
        for (idc, mut timer,  mut sprite) in &mut animatable_sprites {
            timer.tick(time.delta());
            if timer.just_finished() {
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
        server: Res<AssetServer>,
        mut atlases: ResMut<Assets<TextureAtlas>>,
        mut ev_spawn: EventReader<DebugCmdEvent>,
    ) {
        for _e in ev_spawn.iter() {
            match _e.0 {
                DebugCommands::SpawnPlayer => {

                    let texture_handle: Handle<Image> = server.load("main_character_ss.png");

                    let texture_atlas = TextureAtlas::from_grid(
                        texture_handle, Vec2::new(24.0, 24.0),
                        12, 4, None, None
                    );
                    let idc = AnimationIdc{ first: 1, last: 12};
                    let handle = atlases.add(texture_atlas);
                    commands.spawn(
                        (SpriteSheetBundle {
                            texture_atlas: handle,
                            sprite: TextureAtlasSprite::new(0),
                            transform: Transform::from_scale(Vec3::splat(6.0)),
                            ..default()
                        },
                        idc,
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        PlayerPawn
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
        let mut player = match players.get_single_mut() {
            Ok(plyr) => plyr,
            _ => return
        };
        let dt = time.delta_seconds();
        let speed = **speed_res * dt;
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
                Direction::Down => {idc.first = 0; idc.last = 11; sprite.index = idc.first},
                Direction::Up => {idc.first = 12; idc.last = 23; sprite.index = idc.first},
                Direction::Right => {idc.first = 24; idc.last = 35; sprite.index = idc.first},
                Direction::Left => {idc.first = 36; idc.last = 47; sprite.index = idc.first},
            }
        }
    }
}
