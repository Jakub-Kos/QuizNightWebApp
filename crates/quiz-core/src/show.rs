//! Pure show reducer and BroadcastChannel-compatible messages.
use crate::quiz::{
    intro::{prizes_for, rules_for},
    model::{Quiz, Settings},
};
use serde::{Deserialize, Serialize};

/// Screen displayed by both show windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scene {
    /// Opening screen.
    Welcome,
    /// Quiz rules.
    Rules,
    /// Quiz prizes.
    Prizes,
    /// Round selection.
    Dashboard,
    /// Active round.
    Game,
    /// Team scores.
    Leaderboard,
    /// Intermission.
    Pause,
}
/// How a round presents its questions and answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    /// Include answers.
    WithAnswers,
    /// Show questions only.
    QuestionsOnly,
}
/// Identity of a show window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Audience TV.
    Main,
    /// Host controls.
    Presenter,
}
/// State shared between show windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowState {
    /// Current screen.
    pub scene: Scene,
    /// Parsed round's string identifier, retained when leaving a round.
    pub active_round_id: Option<String>,
    /// Current round presentation mode.
    pub play_mode: PlayMode,
}
/// Availability of intro screens.
#[derive(Debug, Clone, Copy)]
pub struct Intro {
    /// Whether rules have content.
    pub has_rules: bool,
    /// Whether prizes are enabled.
    pub has_prizes: bool,
}
impl Intro {
    /// Resolve intro availability using the quiz and translated default rules.
    pub fn new(quiz: &Quiz, default_rules: &[String]) -> Self {
        Self {
            has_rules: rules_for(quiz, default_rules).is_some(),
            has_prizes: prizes_for(quiz).is_some(),
        }
    }
}
/// Local navigation intent.
#[derive(Debug, Clone)]
pub enum Action {
    /// Leave welcome.
    Start,
    /// Leave rules.
    NextRules,
    /// Leave prizes.
    NextPrizes,
    /// Start the selected parsed round in a mode.
    StartRound {
        /// Parsed round identifier.
        round_id: String,
        /// Requested presentation mode.
        mode: PlayMode,
    },
    /// Open scores.
    OpenLeaderboard,
    /// Open intermission.
    OpenPause,
    /// Return via any back button or Shift+D.
    BackToDashboard,
}
/// Wire messages exchanged on quiz-app-sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncMessage {
    /// Shared navigation snapshot.
    StateUpdate {
        /// Sending window.
        sender: Role,
        /// Shared snapshot.
        payload: ShowState,
    },
    /// Updated quiz settings.
    Config {
        /// Sending window.
        sender: Role,
        /// Updated settings.
        payload: Settings,
    },
    /// Request the peer's snapshot.
    RequestState {
        /// Sending window.
        sender: Role,
    },
    /// Presenter heartbeat.
    PresenterHere {
        /// Sending window.
        sender: Role,
    },
    /// Presenter closing notification.
    PresenterGone {
        /// Sending window.
        sender: Role,
    },
}
impl SyncMessage {
    fn sender(&self) -> Role {
        match self {
            Self::StateUpdate { sender, .. }
            | Self::Config { sender, .. }
            | Self::RequestState { sender }
            | Self::PresenterHere { sender }
            | Self::PresenterGone { sender } => *sender,
        }
    }
}
/// One window's pure reducer; transport, persistence and clocks belong to its caller.
pub struct Show {
    role: Role,
    state: ShowState,
    settings: Settings,
    intro: Intro,
    presenter_open: bool,
    last_synced: ShowState,
}
impl Show {
    /// Create a window at welcome with the JS default mode.
    pub fn new(role: Role, settings: Settings, intro: Intro) -> Self {
        let state = ShowState {
            scene: Scene::Welcome,
            active_round_id: None,
            play_mode: PlayMode::WithAnswers,
        };
        Self {
            role,
            last_synced: state.clone(),
            state,
            settings,
            intro,
            presenter_open: false,
        }
    }
    /// Opening request plus the presenter's initial presence announcement, in JS effect order.
    pub fn on_open(&self) -> Vec<SyncMessage> {
        let mut messages = vec![SyncMessage::RequestState { sender: self.role }];
        if self.role == Role::Presenter {
            messages.push(SyncMessage::PresenterHere { sender: self.role });
        }
        messages
    }
    fn snapshot(&self) -> SyncMessage {
        SyncMessage::StateUpdate {
            sender: self.role,
            payload: self.state.clone(),
        }
    }
    /// Apply local navigation and broadcast only a changed shared snapshot.
    pub fn dispatch(&mut self, action: Action) -> Vec<SyncMessage> {
        self.state.scene = match action {
            Action::Start if self.intro.has_rules => Scene::Rules,
            Action::Start | Action::NextRules if self.intro.has_prizes => Scene::Prizes,
            Action::Start | Action::NextRules | Action::NextPrizes | Action::BackToDashboard => {
                Scene::Dashboard
            }
            Action::OpenLeaderboard => Scene::Leaderboard,
            Action::OpenPause => Scene::Pause,
            Action::StartRound { round_id, mode } => {
                self.state.active_round_id = Some(round_id);
                self.state.play_mode = mode;
                Scene::Game
            }
        };
        if self.state == self.last_synced {
            return vec![];
        }
        self.last_synced = self.state.clone();
        vec![self.snapshot()]
    }
    /// Replace settings and always announce CONFIG; caller persists them.
    pub fn set_settings(&mut self, settings: Settings) -> Vec<SyncMessage> {
        self.settings = settings.clone();
        vec![SyncMessage::Config {
            sender: self.role,
            payload: settings,
        }]
    }
    /// Apply peer messages without echoing received state.
    pub fn receive(&mut self, message: &SyncMessage) -> Vec<SyncMessage> {
        if message.sender() == self.role {
            return vec![];
        }
        if message.sender() == Role::Presenter {
            self.presenter_open = !matches!(message, SyncMessage::PresenterGone { .. });
        }
        match message {
            SyncMessage::Config { payload, .. } => self.settings = payload.clone(),
            SyncMessage::StateUpdate { payload, .. } => {
                self.state = payload.clone();
                self.last_synced = payload.clone();
            }
            SyncMessage::RequestState { .. } => return vec![self.snapshot()],
            _ => {}
        }
        vec![]
    }
    /// Window identity.
    pub fn role(&self) -> Role {
        self.role
    }
    /// Shared navigation snapshot.
    pub fn state(&self) -> &ShowState {
        &self.state
    }
    /// Current quiz settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
    /// Last locally broadcast or received navigation snapshot.
    pub fn last_synced(&self) -> &ShowState {
        &self.last_synced
    }
    /// Whether peer presence currently suppresses main-window audio.
    pub fn presenter_open(&self) -> bool {
        self.presenter_open
    }
    /// Update presence on timer expiry; caller owns the timer (JS expires after 30 seconds).
    pub fn note_presenter(&mut self, here: bool) {
        self.presenter_open = here;
    }
    /// Whether this window owns sound playback, independent of pack, volume and enabled settings.
    pub fn plays_sounds(&self) -> bool {
        self.role == Role::Presenter || !self.presenter_open
    }
    /// Whether presenter controls bypass the scaled stage while calibration is inactive.
    pub fn uses_full_window(&self, calibrating: bool) -> bool {
        self.role == Role::Presenter
            && !calibrating
            && matches!(
                self.state.scene,
                Scene::Game | Scene::Leaderboard | Scene::Pause
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn show(role: Role) -> Show {
        Show::new(
            role,
            Settings::default(),
            Intro {
                has_rules: false,
                has_prizes: false,
            },
        )
    }
    #[test]
    fn intro_paths() {
        for rules in [false, true] {
            for prizes in [false, true] {
                let mut quiz = Quiz::default();
                quiz.rules.enabled = rules;
                quiz.prizes.enabled = prizes;
                let intro = Intro::new(&quiz, &["Rule".into()]);
                let mut s = Show::new(Role::Main, Settings::default(), intro);
                s.dispatch(Action::Start);
                if rules {
                    assert_eq!(s.state.scene, Scene::Rules);
                    s.dispatch(Action::NextRules);
                }
                if prizes {
                    assert_eq!(s.state.scene, Scene::Prizes);
                    s.dispatch(Action::NextPrizes);
                }
                assert_eq!(s.state.scene, Scene::Dashboard);
            }
        }
    }
    fn exchange(a: &mut Show, b: &mut Show, messages: Vec<SyncMessage>) {
        let mut queue = std::collections::VecDeque::from(messages);
        let mut count = 0;
        while let Some(message) = queue.pop_front() {
            count += 1;
            assert!(count < 20, "echo loop");
            let target = if message.sender() == a.role {
                &mut *b
            } else {
                &mut *a
            };
            queue.extend(target.receive(&message));
        }
        assert_eq!(a.state(), b.state());
    }
    #[test]
    fn navigation_and_sync() {
        let mut main = show(Role::Main);
        let action = Action::StartRound {
            round_id: "round-1".into(),
            mode: PlayMode::QuestionsOnly,
        };
        let messages = main.dispatch(action.clone());
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], SyncMessage::StateUpdate { .. }));
        assert_eq!(main.state.scene, Scene::Game);
        assert_eq!(main.state.active_round_id.as_deref(), Some("round-1"));
        assert_eq!(main.state.play_mode, PlayMode::QuestionsOnly);
        assert!(main.dispatch(action).is_empty());
        let mut presenter = show(Role::Presenter);
        let opening = presenter.on_open();
        assert_eq!(opening.len(), 2);
        exchange(&mut main, &mut presenter, opening);
        for action in [
            Action::BackToDashboard,
            Action::OpenPause,
            Action::OpenLeaderboard,
            Action::StartRound {
                round_id: "2".into(),
                mode: PlayMode::WithAnswers,
            },
        ] {
            let messages = presenter.dispatch(action.clone());
            exchange(&mut main, &mut presenter, messages);
            let messages = main.dispatch(action);
            exchange(&mut main, &mut presenter, messages);
        }
        let settings = Settings {
            language: "en".into(),
            ..Settings::default()
        };
        let messages = presenter.set_settings(settings.clone());
        exchange(&mut main, &mut presenter, messages);
        assert_eq!(main.settings(), &settings);
        let before = main.state.clone();
        assert!(
            main.receive(&SyncMessage::StateUpdate {
                sender: Role::Main,
                payload: show(Role::Main).state
            })
            .is_empty()
        );
        assert_eq!(main.state, before);
    }
    #[test]
    fn wire_format() {
        let state = show(Role::Main).state;
        let cases = [
            (
                SyncMessage::StateUpdate {
                    sender: Role::Main,
                    payload: state,
                },
                json!({"type":"STATE_UPDATE","sender":"main","payload":{
                    "scene":"welcome","activeRoundId":null,"playMode":"with_answers"}}),
            ),
            (
                SyncMessage::Config {
                    sender: Role::Presenter,
                    payload: Settings::default(),
                },
                json!({"type":"CONFIG","sender":"presenter","payload":Settings::default()}),
            ),
            (
                SyncMessage::RequestState { sender: Role::Main },
                json!({"type":"REQUEST_STATE","sender":"main"}),
            ),
            (
                SyncMessage::PresenterHere {
                    sender: Role::Presenter,
                },
                json!({"type":"PRESENTER_HERE","sender":"presenter"}),
            ),
            (
                SyncMessage::PresenterGone {
                    sender: Role::Presenter,
                },
                json!({"type":"PRESENTER_GONE","sender":"presenter"}),
            ),
        ];
        for (message, expected) in cases {
            assert_eq!(serde_json::to_value(&message).unwrap(), expected);
            assert_eq!(
                serde_json::from_value::<SyncMessage>(expected).unwrap(),
                message
            );
        }
    }
    #[test]
    fn sound_and_stage_tables() {
        for role in [Role::Main, Role::Presenter] {
            let mut s = show(role);
            for present in [false, true] {
                s.note_presenter(present);
                assert_eq!(s.plays_sounds(), role == Role::Presenter || !present);
            }
            for scene in [
                Scene::Welcome,
                Scene::Rules,
                Scene::Prizes,
                Scene::Dashboard,
                Scene::Game,
                Scene::Leaderboard,
                Scene::Pause,
            ] {
                s.state.scene = scene;
                for calibration in [false, true] {
                    assert_eq!(
                        s.uses_full_window(calibration),
                        role == Role::Presenter
                            && !calibration
                            && matches!(scene, Scene::Game | Scene::Leaderboard | Scene::Pause)
                    );
                }
            }
        }
        let mut main = show(Role::Main);
        for message in [
            SyncMessage::PresenterHere {
                sender: Role::Presenter,
            },
            SyncMessage::RequestState {
                sender: Role::Presenter,
            },
            SyncMessage::PresenterGone {
                sender: Role::Presenter,
            },
        ] {
            main.receive(&message);
            assert_eq!(
                main.presenter_open(),
                !matches!(message, SyncMessage::PresenterGone { .. })
            );
        }
    }
}
