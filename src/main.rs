use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((menu::Menu, game::Game, manager::Mgr))
        .run();
}

fn despawn_recursive<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for e in &to_despawn {
        commands.entity(e).despawn_recursive();
    }
}

mod manager {
    use bevy::prelude::*;

    pub struct Mgr;

    #[derive(Component)]
    pub struct Manager;

    impl Plugin for Mgr {
        fn build(&self, app: &mut App) {
            app.add_state::<GameState>()
                .add_systems(OnEnter(GameState::Loading), load_assets)
                .add_systems(
                    Update,
                    check_if_assets_loaded.run_if(in_state(GameState::Loading)),
                );
        }
    }
    #[derive(Resource, Deref, DerefMut)]
    struct FauxLoadingTimer(Timer);

    fn load_assets(mut commands: Commands) {
        commands.insert_resource(FauxLoadingTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    }

    fn check_if_assets_loaded(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<FauxLoadingTimer>,
        mut gamestate: ResMut<NextState<GameState>>,
    ) {
        if timer.tick(time.delta()).finished() {
            gamestate.set(GameState::Menu);
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
                .add_systems(Update, button_system.run_if(in_state(GameState::Menu)));
        }
    }

    fn setup(mut commands: Commands) {
        let bg = NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::AZURE.into(),
            ..default()
        };
        let label = TextBundle::from_section(
            "AutoBattler",
            TextStyle {
                font_size: 80.0,
                color: Color::BLUE.into(),
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
        commands.spawn(Camera2dBundle::default());
        commands
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::Rgba {
                    red: 0.0,
                    green: 0.2,
                    blue: 0.2,
                    alpha: 1.0,
                }
                .into(),
                ..default()
            })
            .with_children(|base| {
                base.spawn((bg, MenuRoot)).with_children(|background| {
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
    use bevy::prelude::*;
    pub struct Game;
    impl Plugin for Game {
        fn build(&self, app: &mut App) {}
    }
}
