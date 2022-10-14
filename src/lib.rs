use std::path::PathBuf;

use tui::widgets::ListState;

pub enum Event {
    Search(String),
    Sort(Option<Sort>),
    Up,
    Down,
    Left,
    Right,
}

#[derive(Eq, PartialEq)]
pub enum Sort {
    Name,
    Size,
    Mtime,
    Ctime,
}

pub struct State {
    dir: PathBuf,
    search: String,
    selected: usize,
    files: Vec<PathBuf>,
    sort: Option<Sort>,
    reverse: bool,
    pub list: ListState,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            dir: std::env::current_dir().unwrap(),
            search: String::new(),
            selected: 0,
            files: Vec::new(),
            sort: None,
            reverse: false,
            list: ListState::default(),
        };
        state.update_files();
        state.list.select(Some(0));
        state
    }
    fn update_files(&mut self) {
        let mut files = self
            .dir
            .read_dir()
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| is_match(path, &self.search))
            .collect::<Vec<_>>();
        if let Some(sort) = &self.sort {
            sort_files(&mut files, sort);
        }
        self.files = files;
    }
    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }
    pub fn get_selected(&self) -> usize {
        self.selected
    }
    pub fn get_current_dir(&self) -> &PathBuf {
        &self.dir
    }
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Search(search) => self.search = search,
            Event::Up => self.selected = (self.selected + self.files.len() - 1) % self.files.len(),
            Event::Down => self.selected = (self.selected + 1) % self.files.len(),
            Event::Left => {
                self.dir.pop();
                self.selected = 0;
            }
            Event::Right => {
                if let Some(file) = self.files.get(self.selected) {
                    if file.is_dir() {
                        self.dir.push(file);
                        self.selected = 0;
                    }
                }
            }
            Event::Sort(sort) => {
                if self.sort == sort {
                    self.reverse = !self.reverse;
                } else {
                    self.sort = sort;
                    self.reverse = false;
                }
            }
        }
        self.update_files();
        self.list.select(Some(self.selected));
    }
}

fn is_match(file: &PathBuf, search: &str) -> bool {
    file.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.contains(search))
        .unwrap_or(false)
}

fn sort_files(files: &mut Vec<PathBuf>, sort: &Sort) {
    match sort {
        Sort::Name => {
            files.sort_by_key(|file| file.file_name().unwrap().to_str().unwrap().to_owned())
        }
        Sort::Size => files.sort_by_key(|file| file.metadata().map(|m| m.len()).unwrap_or(0)),
        Sort::Mtime => files.sort_by_key(|file| {
            file.metadata()
                .unwrap()
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        }),
        Sort::Ctime => files.sort_by_key(|file| {
            file.metadata()
                .unwrap()
                .created()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        }),
    }
}
