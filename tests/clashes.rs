use bevy::prelude::*;
use bevy_ecs::system::SystemState;
use bevy_utils::HashSet;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputStreams;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Action {
    One,
    Two,
    OneAndTwo,
    TwoAndThree,
    OneAndTwoAndThree,
    CtrlOne,
    AltOne,
    CtrlAltOne,
}

fn spawn_input_map(mut commands: Commands) {
    use Action::*;
    use KeyCode::*;

    let mut input_map = InputMap::default();

    input_map.insert(One, Key1);
    input_map.insert(Two, Key2);
    input_map.insert_chord(OneAndTwo, [Key1, Key2]);
    input_map.insert_chord(TwoAndThree, [Key2, Key3]);
    input_map.insert_chord(OneAndTwoAndThree, [Key1, Key2, Key3]);
    input_map.insert_chord(CtrlOne, [LControl, Key1]);
    input_map.insert_chord(AltOne, [LAlt, Key1]);
    input_map.insert_chord(CtrlAltOne, [LControl, LAlt, Key1]);

    commands.spawn().insert(input_map);
}

trait ClashTestExt {
    /// Asserts that the set of `pressed_actions` matches the actions observed
    /// by the entity with the corresponding varaint of the [`ClashStrategy`] enum
    /// in its [`InputMap`] component
    fn assert_input_map_actions_eq(
        &mut self,
        clash_strategy: ClashStrategy,
        pressed_actions: impl IntoIterator<Item = Action>,
    );
}

impl ClashTestExt for App {
    fn assert_input_map_actions_eq(
        &mut self,
        clash_strategy: ClashStrategy,
        pressed_actions: impl IntoIterator<Item = Action>,
    ) {
        let pressed_actions: HashSet<Action> = HashSet::from_iter(pressed_actions.into_iter());
        // SystemState is love, SystemState is life
        let mut input_system_state: SystemState<(Query<&InputMap<Action>>, Res<Input<KeyCode>>)> =
            SystemState::new(&mut self.world);

        let (input_map_query, keyboard) = input_system_state.get(&self.world);

        let input_streams = InputStreams::from_keyboard(&*keyboard);

        let input_map = input_map_query.single();

        let keyboard_input = input_streams.keyboard.unwrap();

        for action in Action::variants() {
            if pressed_actions.contains(&action) {
                assert!(
                    input_map.pressed(action, &input_streams, clash_strategy),
                    "{action:?} was incorrectly not pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}."
                );
            } else {
                assert!(
                    !input_map.pressed(action, &input_streams, clash_strategy),
                    "{action:?} was incorrectly pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}"
                );
            }
        }
    }
}

#[test]
fn input_clash_handling() {
    use bevy_input::InputPlugin;
    use leafwing_input_manager::MockInput;
    use Action::*;
    use KeyCode::*;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_input_map);

    // Two inputs
    app.send_input(Key1);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, Two, OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);

    // Three inputs
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(Key2);
    app.send_input(Key3);
    app.update();

    app.assert_input_map_actions_eq(
        ClashStrategy::PressAll,
        [One, Two, OneAndTwo, TwoAndThree, OneAndTwoAndThree],
    );
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [OneAndTwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);

    // Modifier
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(Key2);
    app.send_input(Key3);
    app.send_input(LControl);
    app.update();

    app.assert_input_map_actions_eq(
        ClashStrategy::PressAll,
        [One, Two, OneAndTwo, TwoAndThree, OneAndTwoAndThree, CtrlOne],
    );
    app.assert_input_map_actions_eq(
        ClashStrategy::PrioritizeLongest,
        [CtrlOne, OneAndTwoAndThree],
    );
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);

    // Multiple modifiers
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(LControl);
    app.send_input(LAlt);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, CtrlOne, AltOne, CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One]);

    // Action order
    app.reset_inputs();
    app.send_input(Key3);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [Two, TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [Two]);
}
