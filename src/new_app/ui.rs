use std::io::Stdout;

use monsim_utils::NOTHING;
use tui::{backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, terminal::CompletedFrame, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap}, Frame, Terminal};

use crate::{debug_to_file, sim::{AvailableActionsForTeam, MessageLog, PartialActionChoice, PerTeam, Renderables, TeamID}};

pub(super) struct Ui<'a> {
    currently_selected_panel: SelectablePanelID,
    message_log_panel: MessageLogPanel,
    active_battler_status_panels: PerTeam<ActiveBattlerStatusPanel>,
    team_status_panels: PerTeam<TeamStatusPanel>,
    action_choice_selection_menus: PerTeam<ActionChoiceSelectionMenu<'a>>,
    selected_choice_indices: PerTeam<Option<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectablePanelID {
    AllyTeamChoiceSelectionMenu,
    MessageLog,
    OpponentTeamChoiceSelectionMenu,
}

struct ActiveBattlerStatusPanel {
    team_name: &'static str,
    active_battler_status: String,
}

struct ActionChoiceSelectionMenu<'a> {
    team_name: &'static str,
    selectable_panel_id: SelectablePanelID,
    action_choice_list: Vec<ListItem<'a>>,
    list_state: ListState,
    currently_selected_choice_index: Option<usize>,
}

struct MessageLogPanel {
    _selectable_panel_id: SelectablePanelID,

    // The index of the first line to be rendered
    scroll_index: usize,
    last_scrollable_line_index: usize,

    previous_message_log_length: usize,
}

struct TeamStatusPanel {
    team_name: &'static str,
    team_status: String,
}

const ALLY_TEAM_NAME: &'static str = "Ally";
const OPPONENT_TEAM_NAME: &'static str = "Opponent";
const MAX_ACTION_CHOICES: usize = 8;

impl<'a> Ui<'a> {
    pub(super) fn new(renderables: &Renderables) -> Self {

        // TODO: There is some redundancy in this lambda with `update_action_choice_list`, but they are slightly different, how to reconcile?
        let get_action_choices_as_vec = |team_id| {
            let available_actions_for_team = renderables.available_actions[team_id];
            let mut action_choice_list = Vec::with_capacity(MAX_ACTION_CHOICES);
            for action_choice in available_actions_for_team {
                match action_choice {
                    PartialActionChoice::Move { display_text, .. } => {
                        action_choice_list.push(ListItem::new(display_text));
                    },
                    PartialActionChoice::SwitchOut { switcher_uid: _ } => {
                        action_choice_list.push(ListItem::new("Switch Out"));
                    },
                }
            }
            action_choice_list
        };

        Self {
            currently_selected_panel: SelectablePanelID::MessageLog,
            active_battler_status_panels: PerTeam::new(
                ActiveBattlerStatusPanel {
                    team_name: ALLY_TEAM_NAME,
                    active_battler_status: renderables.team_status_renderables[TeamID::Allies]
                        .active_battler_status.clone(),
                }, 
                ActiveBattlerStatusPanel {
                    team_name: OPPONENT_TEAM_NAME,
                    active_battler_status: renderables.team_status_renderables[TeamID::Opponents]
                        .active_battler_status.clone(),
                    }
                ),
            action_choice_selection_menus: PerTeam::new(
                    ActionChoiceSelectionMenu { 
                        team_name: ALLY_TEAM_NAME, 
                        selectable_panel_id: SelectablePanelID::AllyTeamChoiceSelectionMenu,
                        action_choice_list: get_action_choices_as_vec(TeamID::Allies),
                        list_state: new_list_state(),
                        currently_selected_choice_index: None,
                    },
                    ActionChoiceSelectionMenu { 
                        team_name: OPPONENT_TEAM_NAME,
                        selectable_panel_id: SelectablePanelID::OpponentTeamChoiceSelectionMenu,
                        action_choice_list: get_action_choices_as_vec(TeamID::Opponents),
                        list_state: new_list_state(),
                        currently_selected_choice_index: None, 
                    },
                ),
            team_status_panels: PerTeam::new(
                TeamStatusPanel {
                    team_name: ALLY_TEAM_NAME,
                    team_status: renderables.team_status_renderables[TeamID::Allies]
                        .team_status.clone(),
                },
                TeamStatusPanel {
                    team_name: OPPONENT_TEAM_NAME,
                    team_status: renderables.team_status_renderables[TeamID::Opponents]
                        .team_status.clone(),
                }, 
            ),
            message_log_panel: MessageLogPanel {
                _selectable_panel_id: SelectablePanelID::MessageLog,
                scroll_index: 0,
                last_scrollable_line_index: 0,
                previous_message_log_length: 0,
            },
            selected_choice_indices: PerTeam::new(None, None),
        }
    }

    pub fn update(&mut self, renderables: &Renderables) {
        self.snap_message_log_to_beginning_of_last_message(renderables.message_log.len());

        self.update_team_status_panel(TeamID::Allies, renderables);
        self.update_team_status_panel(TeamID::Opponents, renderables);
    }

    fn update_team_status_panel(&mut self, team_id: TeamID, renderables: &Renderables) {
        let renderables_for_team = &renderables.team_status_renderables[team_id];
        
        self.action_choice_selection_menus[team_id].update_action_choice_list(renderables.available_actions[team_id]);
        self.active_battler_status_panels[team_id].active_battler_status = renderables_for_team.active_battler_status.clone();
        self.team_status_panels[team_id].team_status = renderables_for_team.team_status.clone();

    }

    pub(super) fn render_to(&self, terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, message_log: &MessageLog) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let chunks = Ui::divide_screen_into_chunks(frame);
            
            // Render the Ally team UI
            let ally_team_panel_chunks = Ui::divide_team_panel_into_chunks(chunks[0]);
            let (ally_team_active_battler_status_widget, ally_team_choice_menu_widget, ally_team_status_widget) = self.renderable_widgets_for_team(TeamID::Allies);

            frame.render_widget(ally_team_active_battler_status_widget, ally_team_panel_chunks[0]);
            // TODO: think about how to remove this clone (and possibly similar ones elsewhere)
            frame.render_stateful_widget(ally_team_choice_menu_widget, ally_team_panel_chunks[1], &mut self.action_choice_selection_menus[TeamID::Allies].list_state.clone());
            frame.render_widget(ally_team_status_widget, ally_team_panel_chunks[2]);

            // Render the message log
            let message_log_widget = self.message_log_panel.as_renderable_widget(message_log, self.currently_selected_panel == SelectablePanelID::MessageLog);
            frame.render_widget(message_log_widget, chunks[1]);

            // Render the Opponent team UI
            let opponent_team_panel_chunks = Ui::divide_team_panel_into_chunks(chunks[2]);
            let (opponent_team_active_battler_status_widget, opponent_team_choice_menu_widget, opponent_team_status_widget) = self.renderable_widgets_for_team(TeamID::Opponents);

            frame.render_widget(opponent_team_active_battler_status_widget, opponent_team_panel_chunks[0]);
            frame.render_stateful_widget(opponent_team_choice_menu_widget, opponent_team_panel_chunks[1], &mut self.action_choice_selection_menus[TeamID::Opponents].list_state.clone());
            frame.render_widget(opponent_team_status_widget, opponent_team_panel_chunks[2]);
        })
    }

    fn renderable_widgets_for_team(&self, team_id: TeamID) -> (Paragraph<'_>, List<'_>, Paragraph<'_>) {
        let active_battler_status_widget = self.active_battler_status_panels[team_id].as_renderable_widget();
        let is_choice_menu_selected = self.currently_selected_panel == self.action_choice_selection_menus[team_id].selectable_panel_id;
        let choice_menu_widget = self.action_choice_selection_menus[team_id].as_renderable_widget(is_choice_menu_selected);
        let team_status_widget = self.team_status_panels[team_id].as_renderable_widget();
        (active_battler_status_widget, choice_menu_widget, team_status_widget)
    }

    fn divide_screen_into_chunks(frame: &mut Frame<CrosstermBackend<Stdout>>) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Min(25),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(frame.size())
    }
    
    fn divide_team_panel_into_chunks(chunk: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(33), Constraint::Length(8), Constraint::Length(6)].as_ref())
            .split(chunk)
    }

    pub(super) fn scroll_current_widget_up(&mut self) {
        match self.currently_selected_panel {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => {
                self.action_choice_selection_menus[TeamID::Allies].scroll_up();
            },
            SelectablePanelID::MessageLog => { self.scroll_message_log_up() },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => {
                self.action_choice_selection_menus[TeamID::Opponents].scroll_up();
            },
        }
    }

    pub(super) fn scroll_current_widget_down(&mut self) {
        match self.currently_selected_panel {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => {
                self.action_choice_selection_menus[TeamID::Allies].scroll_down();
            },
            SelectablePanelID::MessageLog => { self.scroll_message_log_down() },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => {
                self.action_choice_selection_menus[TeamID::Opponents].scroll_down();
            },
        }
    }

    pub(super) fn select_left_widget(&mut self) {
        self.currently_selected_panel.shift_left()
    }

    pub(super) fn select_right_widget(&mut self) {
        self.currently_selected_panel.shift_right()
    }

    pub(super) fn select_currently_hightlighted_choice(&mut self) {
        let maybe_team_id = match self.currently_selected_panel {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => Some(TeamID::Allies),
            SelectablePanelID::MessageLog => None,
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => Some(TeamID::Opponents),
        };
        if let Some(team_id) = maybe_team_id {
            let highlighted_choice_index = self.action_choice_selection_menus[team_id].list_state.selected().expect("This is initialised to Some and never set to None afterwards");
            self.selected_choice_indices[team_id] = Some(highlighted_choice_index);
        }
    }

    /// Only returns `Some` if both teams have made selections
    pub(super) fn selections_if_both_selected(&self) -> Option<PerTeam<usize>> {
        let ally_team_selected_choice_index = self.selected_choice_indices[TeamID::Allies];
        let opponent_team_selected_choice_index = self.selected_choice_indices[TeamID::Opponents];
        
        if let (
            Some(ally_team_selected_choice_index), 
            Some(opponent_team_selected_choice_index)
        ) = (ally_team_selected_choice_index, opponent_team_selected_choice_index) {
            Some(PerTeam::new(ally_team_selected_choice_index, opponent_team_selected_choice_index))
        } else {
            None
        }
    }

    pub(super) fn scroll_message_log_up(&mut self) {
        self.message_log_panel.scroll_up()
    }

    pub(super) fn scroll_message_log_down(&mut self) {
        self.message_log_panel.scroll_down()
    }

    pub(crate) fn snap_message_log_to_beginning_of_last_message(&mut self, new_message_log_length: usize) {
        self.message_log_panel.snap_to_beginning_of_last_message(new_message_log_length);
    }
}

impl SelectablePanelID {
    pub fn shift_right(&mut self) {
        match self {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => { *self = SelectablePanelID::MessageLog },
            SelectablePanelID::MessageLog => { *self = SelectablePanelID::OpponentTeamChoiceSelectionMenu },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => { *self = SelectablePanelID::AllyTeamChoiceSelectionMenu },
            _ => NOTHING, // Shifting right does nothing.
        }
    }

    pub fn shift_left(&mut self) {
        match self {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => { *self = SelectablePanelID::OpponentTeamChoiceSelectionMenu },
            SelectablePanelID::MessageLog => { *self = SelectablePanelID::AllyTeamChoiceSelectionMenu },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => { *self = SelectablePanelID::MessageLog },
            _ => NOTHING, // Shifting right does nothing.
        }
    }
}

impl ActiveBattlerStatusPanel {
    fn as_renderable_widget<'a>(&'a self) -> Paragraph<'a> {
        Paragraph::new(self.active_battler_status.as_str())
            .block(Block::default().title(format![" {team_name} Active Monster ", team_name = self.team_name])
            .borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }
}

impl<'a> ActionChoiceSelectionMenu<'a> {
    fn as_renderable_widget(&self, is_selected: bool) -> List<'a> {
        let list_items = self.action_choice_list.iter().enumerate().map(|(i, list_item)| {
            let is_selected_item = self.currently_selected_choice_index == Some(i);
            if is_selected_item {
                // Colour the selected choice green
                list_item.clone().style(Style::default().fg(Color::Green))
            } else {
                // The rest are left default.
                list_item.clone()
            }
        }).collect::<Vec<_>>();

        List::new(list_items)
            .block(
                Block::default()
                    .title(if is_selected {
                        Span::styled(format![" {team_name} Action Choices ", team_name = self.team_name], Style::default().fg(Color::Yellow))
                    } else {
                        Span::raw(format![" {team_name} Action Choices ", team_name = self.team_name])
                    })
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
    }

    fn scroll_up(&mut self) {
        let currently_highlighted_index = self.list_state.selected().expect("This is initialised to Some and never set to None afterwards");
        let new_highlighted_index = (currently_highlighted_index + self.action_choice_list.len() - 1) % self.action_choice_list.len();
        self.list_state.select(Some(new_highlighted_index));
    }

    fn scroll_down(&mut self) {
        let currently_highlighted_index = self.list_state.selected().expect("This is initialised to Some and never set to None afterwards");
        let new_highlighted_index = (currently_highlighted_index + 1) % self.action_choice_list.len();
        self.list_state.select(Some(new_highlighted_index));
    }

    fn update_action_choice_list(&mut self, available_actions_for_team: AvailableActionsForTeam) {
        self.action_choice_list.clear();
        for action_choice in available_actions_for_team {
            match action_choice {
                PartialActionChoice::Move { display_text, .. } => {
                    self.action_choice_list.push(ListItem::new(display_text));
                },
                PartialActionChoice::SwitchOut { switcher_uid: _ } => {
                    self.action_choice_list.push(ListItem::new("Switch Out"));
                },
            }
        }
    }
}

impl MessageLogPanel {
    fn as_renderable_widget<'a>(&self, message_log: &'a [String], is_selected: bool) -> Paragraph<'a> {
        let text = message_log
            .iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                if idx >= self.scroll_index { 
                    if element.contains("Turn") {
                        Some(Spans::from(Span::styled(element, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
                    } else {
                        Some(Spans::from(Span::raw(element)))
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(if is_selected {
                        Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
                    } else {
                        Span::raw(" Message Log ")
                    })
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
    }
    
    fn scroll_up(&mut self) {
        self.scroll_index = self.scroll_index.saturating_sub(1);
        // TODO: Check if the following line is redundant.
        self.scroll_index = self.scroll_index.min(self.last_scrollable_line_index);
    }

    fn scroll_down(&mut self) {
        self.scroll_index = (self.scroll_index + 1).min(self.previous_message_log_length);
        self.scroll_index = self.scroll_index.min(self.last_scrollable_line_index);
    }

    fn snap_to_beginning_of_last_message(&mut self, new_message_log_length: usize) {
        self.scroll_index = self.previous_message_log_length;
        self.last_scrollable_line_index = self.scroll_index;
        self.previous_message_log_length = new_message_log_length;
    }
}

impl TeamStatusPanel {
    fn as_renderable_widget<'a>(&'a self) -> Paragraph<'a> {
        Paragraph::new(self.team_status.as_str())
                .block(
                    Block::default()
                        .title({
                            Span::raw(format![" {team_name} Team Status ", team_name = self.team_name])
                        })
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
    }
}

fn new_list_state() -> ListState {
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    list_state
}