use procfs::process;
use std::collections::HashMap;
use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::COPY_DEPTH_FROM_PARENT;

#[derive(Debug)]
pub struct XWindow {
    conn: RustConnection,
    root: Window,
    atom: AtomCollection,
    pub windows: HashMap<i32, Window>,
    pub top_pid: Option<i32>,
}

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        _NET_ACTIVE_WINDOW,
        _NET_CLIENT_LIST,
        _NET_WM_PID,
        _NET_CLOSE_WINDOW,
    }
}

impl XWindow {
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let atom = AtomCollection::new(&conn).unwrap().reply().unwrap();
        let windows = HashMap::new();
        XWindow {
            conn,
            root,
            atom,
            windows,
            top_pid: None,
        }
    }

    pub fn update(&mut self) {
        self.windows = self.clients();
        self.top_pid = self.top_pid();
    }

    pub fn is_top(&self, pid: i32) -> bool {
        match self.top_pid {
            Some(top_pid) => {
                let p = process::Process::new(pid);
                let top_p = process::Process::new(top_pid);
                match (p, top_p) {
                    (Ok(proc), Ok(top_proc)) => match (proc.stat(), top_proc.stat()) {
                        (Ok(pstat), Ok(top_pstat)) => pstat.pgrp == top_pstat.pgrp,
                        _ => false,
                    },
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn top_pid(&self) -> Option<i32> {
        self.pid(self.active_window())
    }

    pub fn clients(&self) -> HashMap<i32, Window> {
        let mut clients = HashMap::<i32, Window>::new();
        if let Ok(c) = self.conn.get_property(
            false,
            self.root,
            self.atom._NET_CLIENT_LIST,
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        ) {
            if let Ok(p) = c.reply() {
                if let Some(ws) = p.value32() {
                    for w in ws {
                        if let Some(pid) = self.pid(w) {
                            clients.insert(pid, w);
                        }
                    }
                }
            }
        }
        clients
    }

    fn pid(&self, w: Window) -> Option<i32> {
        match get_property(
            &self.conn,
            false,
            w,
            self.atom._NET_WM_PID,
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        ) {
            Ok(c) => match c.reply() {
                Ok(r) => r.value32().map(|mut prop| prop.next().unwrap() as i32),
                _ => None,
            },
            _ => None,
        }
    }

    fn active_window(&self) -> Window {
        let reply = get_property(
            &self.conn,
            false,
            self.root,
            self.atom._NET_ACTIVE_WINDOW,
            AtomEnum::WINDOW,
            0,
            u32::MAX,
        )
        .unwrap()
        .reply()
        .unwrap();
        let mut prop = reply.value32().unwrap();
        prop.next().unwrap()
    }

    pub fn close_pid(&self, pid: i32) {
        let window = self.windows.get(&pid);
        match window {
            Some(w) => self.close(*w),
            _ => println!("Could not find window of pid {}", pid),
        }
    }

    fn close(&self, w: Window) {
        let msg = ClientMessageEvent::new(
            32,
            w,
            self.atom._NET_CLOSE_WINDOW,
            ClientMessageData::from([0u32; 5]),
        );
        let _ = send_event(
            &self.conn,
            false,
            w,
            EventMask::STRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
            msg,
        );
    }
}
