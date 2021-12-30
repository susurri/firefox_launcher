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
        XWindow { conn, root, atom }
    }

    pub fn top_pid(&self) -> i32 {
        self.pid(self.active_window()) as i32
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
                        clients.insert(self.pid(w) as i32, w);
                    }
                }
            }
        }
        clients
    }

    fn pid(&self, w: Window) -> u32 {
        let reply = get_property(
            &self.conn,
            false,
            w,
            self.atom._NET_WM_PID,
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        )
        .unwrap()
        .reply()
        .unwrap();
        let mut prop = reply.value32().unwrap();
        prop.next().unwrap()
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

    pub fn close(&self, w: Window) {
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
