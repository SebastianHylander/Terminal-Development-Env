use std::{
    io, path::{PathBuf},
};

use ratatui::widgets::FrameExt as _;

use ratatui_explorer::{FileExplorerBuilder, FileExplorer, Theme};

pub struct Explorer {
    root_dir: PathBuf,
    file_explorer: FileExplorer
} 
impl Explorer {
    fn new(dir_path : PathBuf, mut file_explorer : FileExplorer) -> io::Result<Self> {
        let title = String::from(dir_path.clone().file_stem().unwrap().to_str().unwrap());

        let theme = Theme::default().with_title_top(move |_| {format!("{}/",  title.clone()).into()});
        file_explorer.set_theme(theme);
        

        Ok(Self{
            root_dir: dir_path,
            file_explorer: file_explorer
        })
    }

    pub fn new_from_root_dir(dir_path: PathBuf) -> io::Result<Self> {
        if !dir_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "path is not a directory",
            ));
        }
        Self::new(dir_path.clone(), FileExplorerBuilder::build_with_working_dir(dir_path).unwrap())
    }

    pub fn new_from_file(file_path: PathBuf) -> io::Result<Self> {
        if !file_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path is not a file",
            ));
        }
        Self::new(file_path.clone().parent().unwrap().to_path_buf(), FileExplorerBuilder::build_with_working_file(file_path).unwrap())
    }
    
    pub fn draw(self: &Explorer, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        frame.render_widget_ref(self.file_explorer.widget(), area);
    }
}
