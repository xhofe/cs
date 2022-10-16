use std::path::PathBuf;

use tui::widgets::ListState;

pub enum Event {
    Search,
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

pub struct Node {
    pub name: String,
    pub highlights: Vec<usize>,
    pub path: PathBuf,
}

pub struct App {
    dir: PathBuf,
    selected: usize,
    files: Vec<Node>,
    sort: Option<Sort>,
    reverse: bool,
    pub list: ListState,
    pub search_mode: bool,
    pub search: String,
}

impl App {
    pub fn new() -> Self {
        let mut state = Self {
            dir: std::env::current_dir().unwrap(),
            search: String::new(),
            selected: 0,
            files: Vec::new(),
            sort: None,
            reverse: false,
            list: ListState::default(),
            search_mode: true,
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
            .filter_map(|entry| {
                let name = entry.file_name().to_str().unwrap().to_owned();
                is_match(&name, &self.search).and_then(|highlights| {
                    Some(Node {
                        name,
                        highlights,
                        path: entry.path(),
                    })
                })
            })
            .collect::<Vec<_>>();
        if let Some(sort) = &self.sort {
            sort_files(&mut files, sort);
        }
        self.files = files;
    }
    pub fn get_files(&self) -> &[Node] {
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
            Event::Up => self.selected = (self.selected + self.files.len() - 1) % self.files.len(),
            Event::Down => self.selected = (self.selected + 1) % self.files.len(),
            Event::Left => {
                self.dir.pop();
                self.selected = 0;
                self.update_files();
            }
            Event::Right => {
                if let Some(file) = self.files.get(self.selected) {
                    if file.path.is_dir() {
                        self.dir.push(&file.path);
                        self.selected = 0;
                        self.update_files();
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
                self.update_files();
            }
            Event::Search => {
                self.update_files();
            }
        }
        if self.selected >= self.files.len() {
            self.selected = self.files.len().max(1) - 1;
        }
        self.list.select(Some(self.selected));
    }
}

fn is_match(filename: &str, search: &str) -> Option<Vec<usize>> {
    let mut ans = vec![];
    let search_chars = search.to_lowercase().chars().collect::<Vec<_>>();
    let mut j = 0;
    for (i, c) in filename.to_lowercase().chars().enumerate() {
        if j >= search_chars.len() {
            break;
        }
        if c == search_chars[j] {
            ans.push(i);
            j += 1;
        }
    }
    if j == search_chars.len() {
        Some(ans)
    } else {
        None
    }
}

fn sort_files(files: &mut Vec<Node>, sort: &Sort) {
    match sort {
        Sort::Name => files.sort_by_key(|file| file.name.clone()),
        Sort::Size => files.sort_by_key(|file| file.path.metadata().map(|m| m.len()).unwrap_or(0)),
        Sort::Mtime => files.sort_by_key(|file| {
            file.path
                .metadata()
                .unwrap()
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        }),
        Sort::Ctime => files.sort_by_key(|file| {
            file.path
                .metadata()
                .unwrap()
                .created()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        }),
    }
}
