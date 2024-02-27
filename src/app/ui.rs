use std::io::Stdout;

use monsim_utils::{Ally, ArrayOfOptionals, Opponent};
use tui::{backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, terminal::CompletedFrame, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap}, Frame, Terminal};

use crate::sim::{AvailableActionsForTeam, Battle, MonsterUID, PartiallySpecifiedAction, PerTeam, TeamID};

use super::{AppState, InputMode};

pub(super) struct Ui<'a> {
    currently_selected_panel: SelectablePanelID,
    message_log_panel: MessageLogPanel,
    active_monster_status_panels: PerTeam<ActiveMonsterStatusPanel>,
    team_status_panels: PerTeam<TeamStatusPanel>,
    action_choice_selection_menus: PerTeam<ActionChoiceSelectionMenu<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SelectablePanelID {
    AllyTeamChoiceSelectionMenu,
    MessageLog,
    OpponentTeamChoiceSelectionMenu,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActiveMonsterStatusPanel {
    team_name: &'static str,
    active_monster_status: String,
}

#[derive(Debug, Clone)]
struct ActionChoiceSelectionMenu<'a> {
    team_name: &'static str,
    selectable_panel_id: SelectablePanelID,
    action_choice_list: Vec<ListItem<'a>>,
    list_state: ListState,
    selection_cursor: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MessageLogPanel {
    _selectable_panel_id: SelectablePanelID,

    // The index of the first line to be rendered
    scroll_cursor: usize,
    last_scrollable_line_cursor: usize,

    last_message_log_length: usize,
    current_message_log_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SwitcheePrompt;

impl SwitcheePrompt {
    fn as_renderable_widget<'a>(battle: &Battle, possible_switchee_uids: ArrayOfOptionals<MonsterUID, 5>) -> List<'a> {
        let list_items = possible_switchee_uids.into_iter()
            .flatten()
            .map(|monster_uid| {
                ListItem::new(battle.monster(monster_uid).full_name())
            }).collect::<Vec<_>>();

        List::new(list_items)
            .block(
                Block::default()
                    .title(
                        Span::styled("Switchee?", Style::default().fg(Color::Yellow))
                    )
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TeamStatusPanel {
    team_name: &'static str,
    team_status: String,
}

const ALLY_TEAM_NAME: &str = "Ally";
const OPPONENT_TEAM_NAME: &str = "Opponent";
const MAX_ACTION_CHOICES: usize = 8;

impl<'a> Ui<'a> {
    pub(super) fn new(battle: &Battle) -> Self {
     
        let available_actions = battle.available_actions();

        let mut ally_team_action_choice_list = Vec::with_capacity(MAX_ACTION_CHOICES);
        let ally_team_available_actions = available_actions[TeamID::Allies];
        Self::action_choice_list_from_available_actions_for_team(
            &mut ally_team_action_choice_list, 
            ally_team_available_actions
        );
        
        let mut opponent_team_action_choice_list = Vec::with_capacity(MAX_ACTION_CHOICES);
        let opponent_team_available_actions = available_actions[TeamID::Opponents];
        Self::action_choice_list_from_available_actions_for_team(
            &mut opponent_team_action_choice_list,
            opponent_team_available_actions
        );

        Self {
            currently_selected_panel: SelectablePanelID::MessageLog,
            active_monster_status_panels: PerTeam::new(
                Ally::new(ActiveMonsterStatusPanel {
                    team_name: ALLY_TEAM_NAME,
                    active_monster_status: battle.active_monsters_on_team(TeamID::Allies).status_string(),
                }), 
                Opponent::new(ActiveMonsterStatusPanel {
                    team_name: OPPONENT_TEAM_NAME,
                    active_monster_status: battle.active_monsters_on_team(TeamID::Opponents).status_string(),
                    }
                )),
            action_choice_selection_menus: PerTeam::new(
                    Ally::new(ActionChoiceSelectionMenu { 
                        team_name: ALLY_TEAM_NAME, 
                        selectable_panel_id: SelectablePanelID::AllyTeamChoiceSelectionMenu,
                        action_choice_list: ally_team_action_choice_list,
                        list_state: new_list_state(0),
                        selection_cursor: None,
                    }),
                    Opponent::new(ActionChoiceSelectionMenu { 
                        team_name: OPPONENT_TEAM_NAME,
                        selectable_panel_id: SelectablePanelID::OpponentTeamChoiceSelectionMenu,
                        action_choice_list: opponent_team_action_choice_list,
                        list_state: new_list_state(0),
                        selection_cursor: None, 
                    }),
                ),
            team_status_panels: PerTeam::new(
                Ally::new(TeamStatusPanel {
                    team_name: ALLY_TEAM_NAME,
                    team_status: battle.ally_team().team_status_string(),
                }),
                Opponent::new(TeamStatusPanel {
                    team_name: OPPONENT_TEAM_NAME,
                    team_status: battle.opponent_team().team_status_string(),
                }), 
            ),
            message_log_panel: MessageLogPanel {
                _selectable_panel_id: SelectablePanelID::MessageLog,
                scroll_cursor: 0,
                last_scrollable_line_cursor: 0,
                last_message_log_length: 0,
                current_message_log_length: battle.message_log.len(),
            },
        }
    }

    pub(super) fn update_message_log(&mut self, new_length: usize) {
        self.message_log_panel.update_length(new_length);
        self.snap_message_log_cursor_to_beginning_of_last_message();
    }

    pub(super) fn update_team_status_panels(&mut self, battle: &Battle) {
        self.update_team_status_panel(TeamID::Allies, battle);
        self.update_team_status_panel(TeamID::Opponents, battle);
    }

    fn update_team_status_panel(&mut self, team_id: TeamID, battle: &Battle) {
        
        Self::action_choice_list_from_available_actions_for_team(
            &mut self.action_choice_selection_menus[team_id].action_choice_list, 
            battle.available_actions()[team_id]
        );
        self.active_monster_status_panels[team_id].active_monster_status = battle.active_monsters_on_team(team_id).status_string();
        self.team_status_panels[team_id].team_status = battle.team(team_id).team_status_string();
    }

    pub(super) fn render(
        &self, terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, 
        battle: &Battle,
        current_app_state: &super::AppState,
    ) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let chunks = Ui::divide_screen_into_chunks(frame);
            
            // Render the Ally team UI
            let ally_team_panel_chunks = Ui::divide_team_panel_into_chunks(chunks[0]);
            let (
                ally_team_active_monster_status_widget,
                ally_team_choice_menu_widget, 
                ally_team_status_widget
            ) = self.renderable_widgets_for_team(TeamID::Allies);

            frame.render_widget(ally_team_active_monster_status_widget, ally_team_panel_chunks[0]);
            // TODO: think about how to remove this clone (and possibly similar ones elsewhere)
            frame.render_stateful_widget(ally_team_choice_menu_widget, ally_team_panel_chunks[1], &mut self.action_choice_selection_menus[TeamID::Allies].list_state.clone());
            frame.render_widget(ally_team_status_widget, ally_team_panel_chunks[2]);

            let message_log_widget = self.message_log_panel.as_renderable_widget(&battle.message_log, self.currently_selected_panel == SelectablePanelID::MessageLog);
            // If we are on the SwitcheePrompt, then show the SwitcheePrompt widget
            if let AppState::AcceptingInput(InputMode::SwitcheePrompt {
                possible_switchee_uids,
                highlight_cursor,
                ..
            }) = *current_app_state {
                let switchee_prompt_widget = SwitcheePrompt::as_renderable_widget(battle, possible_switchee_uids);
                let middle_third_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Max(7),
                        Constraint::Percentage(80),
                    ].as_ref(),
                    )
                    .split(chunks[1]);
                let mut switchee_prompt_list_state = new_list_state(highlight_cursor);
                
                //Render the message log and switchee prompt
                frame.render_stateful_widget(switchee_prompt_widget, middle_third_chunks[0], &mut switchee_prompt_list_state);
                frame.render_widget(message_log_widget, middle_third_chunks[1]);
            } else {
                // Render the message log
                frame.render_widget(message_log_widget, chunks[1]);
            }

            // Render the Opponent team UI
            let opponent_team_panel_chunks = Ui::divide_team_panel_into_chunks(chunks[2]);
            let (
                opponent_team_active_monster_status_widget,
                opponent_team_choice_menu_widget, 
                opponent_team_status_widget
            ) = self.renderable_widgets_for_team(TeamID::Opponents);

            frame.render_widget(opponent_team_active_monster_status_widget, opponent_team_panel_chunks[0]);
            frame.render_stateful_widget(opponent_team_choice_menu_widget, opponent_team_panel_chunks[1], &mut self.action_choice_selection_menus[TeamID::Opponents].list_state.clone());
            frame.render_widget(opponent_team_status_widget, opponent_team_panel_chunks[2]);

            
        })
    }

    fn renderable_widgets_for_team(&self, team_id: TeamID) -> (Paragraph<'_>, List<'_>, Paragraph<'_>) {
        let active_monster_status_widget = self.active_monster_status_panels[team_id].as_renderable_widget();
        let is_choice_menu_selected = self.currently_selected_panel == self.action_choice_selection_menus[team_id].selectable_panel_id;
        let choice_menu_widget = self.action_choice_selection_menus[team_id].as_renderable_widget(is_choice_menu_selected);
        let team_status_widget = self.team_status_panels[team_id].as_renderable_widget();
        (active_monster_status_widget, choice_menu_widget, team_status_widget)
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

    pub(super) fn scroll_up_wrapped(cursor: &mut usize, list_length: usize) {
        *cursor = (*cursor + list_length - 1) % list_length 
    }
    
    pub(super) fn scroll_down_wrapped(cursor: &mut usize, list_length: usize) {
        *cursor = (*cursor + 1) % list_length 
    }

    /// Returns the index and `TeamID` of the selected choice if one was successfully selected.
    pub(super) fn select_currently_hightlighted_menu_item(&mut self) -> Option<(usize, TeamID)> {
        let maybe_team_id = match self.currently_selected_panel {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => Some(TeamID::Allies),
            SelectablePanelID::MessageLog => None,
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => Some(TeamID::Opponents),
        };
        if let Some(team_id) = maybe_team_id {
            let highlighted_choice_index = self.action_choice_selection_menus[team_id].list_state.selected().expect("This is initialised to Some and never set to None afterwards");
            self.action_choice_selection_menus[team_id].selection_cursor = Some(highlighted_choice_index);
            Some((highlighted_choice_index, team_id))
        } else {
            None
        }
    }

    pub(super) fn scroll_message_log_up(&mut self) {
        self.message_log_panel.scroll_up();
    }

    pub(super) fn scroll_message_log_down(&mut self) {
        self.message_log_panel.scroll_down();
    }

    pub(crate) fn snap_message_log_cursor_to_beginning_of_last_message(&mut self) {
        self.message_log_panel.snap_to_beginning_of_last_message();
    }

    fn action_choice_list_from_available_actions_for_team(list_to_fill: &mut Vec<ListItem>, available_actions_for_team: AvailableActionsForTeam) {
        list_to_fill.clear();
        for action_choice in available_actions_for_team.as_vec() {
            match action_choice {
                PartiallySpecifiedAction::Move { display_text, .. } => {
                    list_to_fill.push(ListItem::new(display_text));
                },
                PartiallySpecifiedAction::SwitchOut { .. } => {
                    list_to_fill.push(ListItem::new("Switch Out"));
                },
            }
        }
    }

    pub(super) fn clear_choice_menu_selection_for_team(&mut self, team_id: TeamID) {
        self.action_choice_selection_menus[team_id].list_state.select(Some(0));
        self.action_choice_selection_menus[team_id].selection_cursor = None;
    }
}

impl SelectablePanelID {
    pub fn shift_right(&mut self) {
        match self {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => { *self = SelectablePanelID::MessageLog },
            SelectablePanelID::MessageLog => { *self = SelectablePanelID::OpponentTeamChoiceSelectionMenu },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => { *self = SelectablePanelID::AllyTeamChoiceSelectionMenu },
        }
    }

    pub fn shift_left(&mut self) {
        match self {
            SelectablePanelID::AllyTeamChoiceSelectionMenu => { *self = SelectablePanelID::OpponentTeamChoiceSelectionMenu },
            SelectablePanelID::MessageLog => { *self = SelectablePanelID::AllyTeamChoiceSelectionMenu },
            SelectablePanelID::OpponentTeamChoiceSelectionMenu => { *self = SelectablePanelID::MessageLog },
        }
    }
}

impl ActiveMonsterStatusPanel {
    fn as_renderable_widget(&self) -> Paragraph<'_> {
        Paragraph::new(self.active_monster_status.as_str())
            .block(Block::default().title(format![" {team_name} Active Monster ", team_name = self.team_name])
            .borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }
}

impl<'a> ActionChoiceSelectionMenu<'a> {
    fn as_renderable_widget(&self, is_selected: bool) -> List<'a> {
        let list_items = self.action_choice_list.iter().enumerate().map(|(i, list_item)| {
            let is_selected_item = self.selection_cursor == Some(i);
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
}

impl MessageLogPanel {
    fn as_renderable_widget<'a>(&self, message_log: &'a [String], is_selected: bool) -> Paragraph<'a> {
        let text = message_log
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                if index >= self.scroll_cursor {
                    // FIXME: Possible bug that will highlight any message with "Turn" in Cyan. Most problematic for messages involving moves like "U-Turn" or "Flip Turn". 
                    if element.contains("Turn") {
                        Some(Spans::from(Span::styled(element, Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                        )))
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
        self.scroll_cursor = self.scroll_cursor.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        self.scroll_cursor = (self.scroll_cursor + 1).min(self.current_message_log_length);
        self.scroll_cursor = self.scroll_cursor.min(self.last_scrollable_line_cursor);
    }

    fn snap_to_beginning_of_last_message(&mut self) {
        self.scroll_cursor = self.last_message_log_length;
        // The end of the scrollable segment should be the beginning of the messages for the most recent turn calculated.
        self.last_scrollable_line_cursor = self.scroll_cursor;
    }

    fn update_length(&mut self, new_length: usize) {
        self.last_message_log_length = self.current_message_log_length;
        self.current_message_log_length = new_length;
    }
}

impl TeamStatusPanel {
    fn as_renderable_widget(&self) -> Paragraph<'_> {
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

fn new_list_state(initial_selection: usize) -> ListState {
    let mut list_state = ListState::default();
    list_state.select(Some(initial_selection));
    list_state
}