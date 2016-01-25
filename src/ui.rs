
use ncurses::*;
use ::store::PassStore;

pub struct StoreUi<'a> {
    store:      &'a PassStore,
    win_filter: WINDOW,
    win_list:   WINDOW,
    win_show:   WINDOW,
    screen_height: i32,
    screen_width: i32,
}

impl<'a> StoreUi<'a> {
    pub fn new_with_store(store: &PassStore) -> StoreUi {
        initscr();
        noecho();

        let mut h = 0;
        let mut w = 0;
        getmaxyx(stdscr, &mut h, &mut w);

        let mut win_filter = derwin(stdscr, 1, w, 0, 0);
        let mut win_list = derwin(stdscr, 20, w, 1, 0);
        let mut win_show = derwin(stdscr, 5, w, 22, 0);

        StoreUi {
            store: store,
            win_filter: win_filter,
            win_list: win_list,
            win_show: win_show,
            screen_height: h,
            screen_width: w,
        }
    }

    pub fn initialize(&mut self) {
        scrollok(self.win_list, true);
    }

    pub fn list(&mut self) {
        for entry in self.store.entries() {
            wprintw(self.win_list, &format!("{}\n", entry.name()));
        }
        wrefresh(self.win_list);

        let mut ch = wgetch(self.win_show);
        //while ch != KEY_ENTER {
            //match ch {
                //KEY_UP => {
                    //scrl(-1);
                //},
                //KEY_DOWN => {
                    //scrl(1);
                //},
                //_ => ()
            //}
            //refresh();
            //ch = getch();
        //}
    }
}

impl<'a> Drop for StoreUi<'a> {
    fn drop(&mut self) {
        endwin();
    }
}
