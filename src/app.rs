use anyhow::{Context, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::{ScrollbarState, TableState},
};

use crate::component::{EditingInput, InputArena, MainInput};
use crate::model::{Header, Model};

const ITEM_HEIGHT: usize = 4;
pub enum CurrentScreen {
    Main(MainInput),
    Editing(EditingInput),
    Exiting,
}

pub struct AppState {
    pub items: Vec<Header>, // list of all item names found in the SQLite DB
    pub cached: Option<(usize, String)>, // cached value for the UI
    pub selected_index: usize, // current state of the TableState, can be derived from state but used to simplified processes

    // filtering-specific state
    pub filtered_indexes: Vec<usize>,

    // UI-specific state
    pub state: TableState,             // state of the Table that hold items
    pub scroll_state: ScrollbarState,  // state for the scrollbar, synced to the tablestate
    pub current_screen: CurrentScreen, // to know how which screen the ui is focusing
}

impl AppState {
    pub fn new(model: &Model) -> Result<Self> {
        let protos = model.query_protos()?;
        let scrollbar_state = ScrollbarState::new((protos.len() - 1) * ITEM_HEIGHT);
        Ok(AppState {
            items: protos,
            cached: None,
            selected_index: 0,
            filtered_indexes: Vec::new(),
            state: TableState::default().with_selected(0),
            scroll_state: scrollbar_state,
            current_screen: CurrentScreen::Main(MainInput::None),
        })
    }

    pub fn refresh(&mut self, model: &Model) -> Result<()> {
        self.items = model.query_protos()?;
        Ok(())
    }

    pub fn matches_filter(&self, v: &str, f: &str) -> bool {
        v.contains(f)
    }

    pub fn filter(&mut self, filter_value: &str) -> Result<()> {
        if filter_value.is_empty() {
            self.filtered_indexes = (0..self.items.len()).collect();
        } else {
            self.filtered_indexes = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, h)| self.matches_filter(&h.name, filter_value))
                .map(|(i, _)| i)
                .collect();

            //self.update_state(new_state);
        }

        Ok(())
    }

    pub fn get_filtered_data(&self) -> Result<Vec<&Header>> {
        Ok(self
            .filtered_indexes
            .iter()
            .filter_map(|i| self.items.get(*i))
            .collect())
    }

    pub fn get_data(&mut self, model: &Model) -> Result<()> {
        if self.filtered_indexes.is_empty() {
            self.cached = None;
        } else {
            let real_index = self
                .filtered_indexes
                .get(self.selected_index)
                .unwrap_or(self.filtered_indexes.last().unwrap());
            let item = self
                .items
                .get(*real_index)
                .context(format!("Cannot find item from index {0}", real_index))?;

            if let Some((cached_index, _)) = &self.cached {
                if item.rowid == *cached_index {
                    return Ok(());
                }
            }

            self.cached = Some((item.rowid, model.query_data(&item.rowid)?));
        }
        Ok(())
    }

    pub fn update_state(&mut self, new_state: usize) {
        self.selected_index = new_state;
        self.state.select(Some(new_state));
        self.scroll_state = self.scroll_state.position(new_state * ITEM_HEIGHT);
    }

    pub fn next_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_indexes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.update_state(i);

        Ok(())
    }

    pub fn previous_row(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_indexes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.update_state(i);

        Ok(())
    }
}

pub struct App {
    pub model: Model, // file and sqlite db manipulation
    pub state: AppState,
    pub input_arena: InputArena,
    pub exit: bool, // used to terminate the program
}

impl App {
    pub fn new(db_path: std::path::PathBuf, layer_path: std::path::PathBuf) -> Result<Self> {
        let model = Model::new(&db_path, layer_path)?;
        let state = AppState::new(&model)?;

        Ok(App {
            model,
            state,
            input_arena: InputArena::new()?,
            exit: false,
        })
    }

    pub fn toggle_editing(&mut self) {
        if let CurrentScreen::Editing(focused) = &self.state.current_screen {
            match focused {
                EditingInput::Key => {
                    self.state.current_screen = CurrentScreen::Editing(EditingInput::Value)
                }
                EditingInput::Value => {
                    self.state.current_screen = CurrentScreen::Editing(EditingInput::Key)
                }
            };
        } else {
            self.state.current_screen = CurrentScreen::Editing(EditingInput::Value);
        }
    }

    fn handle_key_event_main_screen(
        &mut self,
        key_event: KeyEvent,
        focused: &MainInput,
    ) -> Result<()> {
        match focused {
            MainInput::None => {
                match key_event.code {
                    KeyCode::Char('e') => {
                        self.state.current_screen = CurrentScreen::Editing(EditingInput::Key);
                    }
                    KeyCode::Char('q') => {
                        self.state.current_screen = CurrentScreen::Exiting;
                    }
                    KeyCode::Char('f') => {
                        self.state.current_screen = CurrentScreen::Main(MainInput::Filter)
                    }
                    KeyCode::Down => self.state.next_row()?,
                    KeyCode::Up => self.state.previous_row()?,
                    KeyCode::Char('r') => self.state.refresh(&self.model)?,
                    _ => {}
                };
            }
            MainInput::Filter => {
                match key_event.code {
                    KeyCode::Backspace => {
                        self.input_arena.value_pop(focused.try_into()?)?;
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        self.state.current_screen = CurrentScreen::Main(MainInput::None)
                    }
                    KeyCode::Char(value) => {
                        self.input_arena.value_push(focused.try_into()?, value)?;
                    }
                    _ => {}
                };
            }
        }

        Ok(())
    }

    fn handle_key_event_exit_screen(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('y') => {
                self.exit = true;
            }
            KeyCode::Char('n') | KeyCode::Char('q') => {
                self.state.current_screen = CurrentScreen::Main(MainInput::Filter);
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event_edit_screen(
        &mut self,
        key_event: KeyEvent,
        focused: &EditingInput,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Enter => self.toggle_editing(),
            KeyCode::Backspace => {
                self.input_arena.value_pop(focused.into())?;
            }
            KeyCode::Esc => {
                self.state.current_screen = CurrentScreen::Main(MainInput::None);
            }
            KeyCode::Tab => {
                self.toggle_editing();
            }
            KeyCode::Char(value) => {
                self.input_arena.value_push(focused.into(), value)?;
            }
            _ => {}
        };

        Ok(())
    }

    // THE update function
    pub fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match &self.state.current_screen {
                    CurrentScreen::Main(focused) => {
                        self.handle_key_event_main_screen(key_event, &focused.clone())?
                    }
                    CurrentScreen::Exiting => self.handle_key_event_exit_screen(key_event)?,
                    CurrentScreen::Editing(focused) => {
                        self.handle_key_event_edit_screen(key_event, &focused.clone())?
                    }
                }
            }
            _ => {}
        };

        self.state.filter(
            self.input_arena
                .get_content(&MainInput::Filter.try_into()?)?,
        )?;
        self.state.get_data(&self.model)?;

        Ok(())
    }
}
