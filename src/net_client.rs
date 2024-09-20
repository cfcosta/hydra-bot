use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::{bot::*, net_packet::NetPacket, net_structs::*};

const NET_MAGIC_NUMBER: u32 = 1454104972;

pub struct NetClient {
    socket: UdpSocket,
    state: ClientState,
    server_addr: SocketAddr,
    settings: Option<GameSettings>,
    reject_reason: Option<String>,
    player_name: String,
    drone: bool,
    recv_window_start: u32,
    recv_window: Vec<NetServerRecv>,
    send_queue: Vec<NetServerSend>,
    need_acknowledge: bool,
    gamedata_recv_time: Instant,
    last_latency: i32,
    net_local_wad_sha1sum: [u8; 20],
    net_local_deh_sha1sum: [u8; 20],
    net_local_is_freedoom: bool,
    net_waiting_for_launch: bool,
    net_client_connected: bool,
    net_client_received_wait_data: bool,
    net_client_wait_data: NetWaitData,
    last_send_time: Instant,
    last_ticcmd: TicCmd,
    recvwindow_cmd_base: Vec<TicCmd>,
    bot: Bot,
}

impl NetClient {
    pub fn new(player_name: String, drone: bool) -> Self {
        NetClient {
            socket: UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket"),
            state: ClientState::Disconnected,
            server_addr: "127.0.0.1:2342".parse().unwrap(), // Placeholder
            settings: None,
            reject_reason: None,
            player_name,
            drone,
            recv_window_start: 0,
            recv_window: vec![NetServerRecv::default(); BACKUPTICS],
            send_queue: vec![NetServerSend::default(); BACKUPTICS],
            need_acknowledge: false,
            gamedata_recv_time: Instant::now(),
            last_latency: 0,
            net_local_wad_sha1sum: [0; 20],
            net_local_deh_sha1sum: [0; 20],
            net_local_is_freedoom: false,
            net_waiting_for_launch: false,
            net_client_connected: false,
            net_client_received_wait_data: false,
            net_client_wait_data: NetWaitData::default(),
            last_send_time: Instant::now(),
            last_ticcmd: TicCmd::default(),
            recvwindow_cmd_base: vec![TicCmd::default(); NET_MAXPLAYERS],
            bot: Bot::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.net_client_connected
    }

    pub fn init(&mut self) {
        self.bot.init();
        self.net_client_connected = false;
        self.net_client_received_wait_data = false;
        self.net_waiting_for_launch = false;

        // Try to set player name from environment variables or command line arguments
        if self.player_name.is_empty() {
            self.player_name = std::env::args().nth(1).unwrap_or_else(|| {
                std::env::var("USER")
                    .or_else(|_| std::env::var("USERNAME"))
                    .unwrap_or_else(|_| NetClient::get_random_pet_name())
            });
        }
    }

    fn get_random_pet_name() -> String {
        let pet_names = ["Fluffy", "Buddy", "Max", "Charlie", "Lucy", "Bailey"];
        pet_names[rand::random::<usize>() % pet_names.len()].to_string()
    }

    pub fn run(&mut self) {
        self.run_bot();

        if !self.net_client_connected {
            return;
        }

        self.receive_packets();

        if self.state == ClientState::Disconnected {
            self.handle_disconnected();
        }

        if self.state == ClientState::InGame {
            self.advance_window();
            self.check_resends();
        }

        self.net_waiting_for_launch = self.state == ClientState::WaitingLaunch;
    }

    fn receive_packets(&mut self) {
        self.socket
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");

        loop {
            match NetPacket::receive(&self.socket) {
                Ok((mut packet, addr)) => {
                    if addr == self.server_addr {
                        self.parse_packet(&mut packet);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving packet: {}", e);
                    break;
                }
            }
        }

        self.socket
            .set_nonblocking(false)
            .expect("Failed to set blocking");
    }

    fn handle_disconnected(&mut self) {
        self.receive_tic(
            &[TicCmd::default(); NET_MAXPLAYERS],
            &[false; NET_MAXPLAYERS],
        );
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if self.net_client_connected {
            self.send_disconnect();
        }
        self.state = ClientState::Disconnected;
        self.net_client_connected = false;
    }

    fn parse_packet(&mut self, packet: &mut NetPacket) {
        let packet_type = packet.read_u16().unwrap();

        match NetPacketType::try_from(packet_type) {
            Ok(NetPacketType::Syn) => self.parse_syn(packet),
            Ok(NetPacketType::Rejected) => self.parse_reject(packet),
            Ok(NetPacketType::WaitingData) => self.parse_waiting_data(packet),
            Ok(NetPacketType::Launch) => self.parse_launch(packet),
            Ok(NetPacketType::GameStart) => self.parse_game_start(packet),
            Ok(NetPacketType::GameData) => self.parse_game_data(packet),
            Ok(NetPacketType::GameDataResend) => self.parse_resend_request(packet),
            Ok(NetPacketType::ConsoleMessage) => self.parse_console_message(packet),
            _ => println!("Unknown packet type: {}", packet_type),
        }
    }

    fn parse_syn(&mut self, packet: &mut NetPacket) {
        println!("Client: Processing SYN response");
        let server_version = packet.read_string().unwrap_or_default();

        println!("Client: Connected to server");
        self.state = ClientState::WaitingLaunch;

        if server_version != env!("CARGO_PKG_VERSION") {
            println!(
                "Client: Warning: This is '{}', but the server is '{}'. \
                It is possible that this mismatch may cause the game to desynchronize.",
                env!("CARGO_PKG_VERSION"),
                server_version
            );
        }
    }

    fn parse_reject(&mut self, packet: &mut NetPacket) {
        if let Some(msg) = packet.read_string() {
            if self.state == ClientState::WaitingLaunch {
                self.state = ClientState::Disconnected;
                self.reject_reason = Some(msg);
            }
        }
    }

    fn parse_waiting_data(&mut self, packet: &mut NetPacket) {
        if let Some(wait_data) = packet.read_wait_data() {
            if wait_data.num_players > wait_data.max_players
                || wait_data.ready_players > wait_data.num_players
                || wait_data.max_players > NET_MAXPLAYERS as i32
            {
                return;
            }

            if (wait_data.consoleplayer >= 0 && self.drone)
                || (wait_data.consoleplayer < 0 && !self.drone)
                || (wait_data.consoleplayer as usize >= wait_data.num_players as usize)
            {
                return;
            }

            self.net_client_wait_data = wait_data;
            self.net_client_received_wait_data = true;
        }
    }

    fn parse_launch(&mut self, packet: &mut NetPacket) {
        println!("Client: Processing launch packet");
        if self.state != ClientState::WaitingLaunch {
            println!("Client: Error: Not in waiting launch state");
            return;
        }

        if let Some(num_players) = packet.read_u8() {
            self.net_client_wait_data.num_players = num_players as i32;
            self.state = ClientState::WaitingStart;
            println!("Client: Now waiting to start the game");
        }
    }

    fn parse_game_start(&mut self, packet: &mut NetPacket) {
        println!("Client: Processing game start packet");
        if let Some(settings) = packet.read_settings() {
            if self.state != ClientState::WaitingStart {
                println!("Client: Error: Not in waiting start state");
                return;
            }

            if settings.num_players > NET_MAXPLAYERS as i32
                || settings.consoleplayer as usize >= settings.num_players as usize
            {
                println!(
                    "Client: Error: Invalid settings, num_players={}, consoleplayer={}",
                    settings.num_players, settings.consoleplayer
                );
                return;
            }

            if (self.drone && settings.consoleplayer >= 0)
                || (!self.drone && settings.consoleplayer < 0)
            {
                println!(
                    "Client: Error: Mismatch: drone={}, consoleplayer={}",
                    self.drone, settings.consoleplayer
                );
                return;
            }

            println!("Client: Initiating game state");
            self.state = ClientState::InGame;
            self.settings = Some(settings);
            self.recv_window_start = 0;
            self.recv_window = vec![NetServerRecv::default(); BACKUPTICS];
            self.send_queue = vec![NetServerSend::default(); BACKUPTICS];
        }
    }

    fn parse_game_data(&mut self, packet: &mut NetPacket) {
        println!("Client: Processing game data packet");

        if let (Some(seq), Some(num_tics)) = (packet.read_u8(), packet.read_u8()) {
            let seq = self.expand_tic_num(seq as u32);
            println!(
                "Client: Game data received, seq={}, num_tics={}",
                seq, num_tics
            );

            let lowres_turn = self.settings.as_ref().unwrap().lowres_turn != 0;

            for i in 0..num_tics {
                if let Some(cmd) = packet.read_full_ticcmd(lowres_turn) {
                    let index = (seq + i as u32 - self.recv_window_start) as usize;
                    if index < BACKUPTICS {
                        self.recv_window[index].active = true;
                        self.recv_window[index].cmd = cmd;
                        println!("Client: Stored tic {} in receive window", seq + i as u32);
                        if i == num_tics - 1 {
                            self.update_clock_sync(seq + i as u32, cmd.latency);
                        }
                    }
                }
            }

            self.need_acknowledge = true;
            self.gamedata_recv_time = Instant::now();

            // Check for missing tics and request resends
            let resend_end = seq as i32 - self.recv_window_start as i32;
            if resend_end > 0 {
                let mut resend_start = resend_end - 1;
                while resend_start >= 0 && !self.recv_window[resend_start as usize].active {
                    resend_start -= 1;
                }
                if resend_start < resend_end - 1 {
                    self.send_resend_request(
                        self.recv_window_start + resend_start as u32 + 1,
                        self.recv_window_start + resend_end as u32 - 1,
                    );
                }
            }
        }
    }

    fn parse_resend_request(&mut self, packet: &mut NetPacket) {
        println!("Client: Processing resend request");
        if self.drone {
            println!("Client: Error: Resend request but we are a drone");
            return;
        }

        if let (Some(start), Some(num_tics)) = (packet.read_i32(), packet.read_u8()) {
            let end = start + num_tics as i32 - 1;
            println!(
                "Client: Resend request: start={}, num_tics={}",
                start, num_tics
            );

            let mut resend_start = start as u32;
            let mut resend_end = end as u32;

            while resend_start <= resend_end
                && (!self.send_queue[resend_start as usize % BACKUPTICS].active
                    || self.send_queue[resend_start as usize % BACKUPTICS].seq != resend_start)
            {
                resend_start += 1;
            }

            while resend_start <= resend_end
                && (!self.send_queue[resend_end as usize % BACKUPTICS].active
                    || self.send_queue[resend_end as usize % BACKUPTICS].seq != resend_end)
            {
                resend_end -= 1;
            }

            if resend_start <= resend_end {
                println!("Client: Resending tics {}-{}", resend_start, resend_end);
                self.send_tics(resend_start, resend_end);
            } else {
                println!("Client: Don't have the tics to resend");
            }
        }
    }

    fn parse_console_message(&self, packet: &mut NetPacket) {
        if let Some(msg) = packet.read_string() {
            println!("Message from server:\n{}", msg);
        }
    }

    fn expand_tic_num(&self, b: u32) -> u32 {
        let l = self.recv_window_start & 0xff;
        let h = self.recv_window_start & !0xff;
        let mut result = h | b;

        if l < 0x40 && b > 0xb0 {
            result = result.wrapping_sub(0x100);
        }
        if l > 0xb0 && b < 0x40 {
            result = result.wrapping_add(0x100);
        }

        result
    }

    fn update_clock_sync(&mut self, seq: u32, remote_latency: i32) {
        const KP: f32 = 0.1;
        const KI: f32 = 0.01;
        const KD: f32 = 0.02;

        let latency = self.send_queue[seq as usize % BACKUPTICS]
            .time
            .elapsed()
            .as_millis() as i32;
        let error = latency - remote_latency;

        // Update PID variables (these should be stored in the struct)
        let mut cumul_error = 0;
        let mut last_error = 0;

        cumul_error += error;
        let offset_ms =
            (KP * error as f32 - KI * cumul_error as f32 + KD * (last_error - error) as f32) as i32;

        last_error = error;
        self.last_latency = latency;

        println!(
            "Client: Latency {}, remote {}, offset={}ms, cumul_error={}",
            latency, remote_latency, offset_ms, cumul_error
        );
    }

    fn send_resend_request(&mut self, start: u32, end: u32) {
        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::GameDataResend as u16);
        packet.write_i32(start as i32);
        packet.write_u8((end - start + 1) as u8);

        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send packet");

        let now = Instant::now();
        for i in start..=end {
            let index = (i - self.recv_window_start) as usize;
            if index < BACKUPTICS {
                self.recv_window[index].resend_time = now;
            }
        }
    }

    fn send_game_data_ack(&mut self) {
        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::GameDataAck as u16);
        packet.write_u8((self.recv_window_start & 0xff) as u8);

        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send packet");
        self.need_acknowledge = false;
        println!("Client: Game data acknowledgment sent");
    }

    fn send_tics(&mut self, start: u32, end: u32) {
        if !self.net_client_connected {
            return;
        }

        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::GameData as u16);
        packet.write_u8((self.recv_window_start & 0xff) as u8);
        packet.write_u8((start & 0xff) as u8);
        packet.write_u8(((end - start + 1) & 0xff) as u8);

        for tic in start..=end {
            if let Some(send_obj) = self.send_queue.get(tic as usize % BACKUPTICS) {
                packet.write_i16(self.last_latency.try_into().unwrap());
                packet.write_ticcmd_diff(
                    &send_obj.cmd,
                    self.settings.as_ref().unwrap().lowres_turn != 0,
                );
            }
        }

        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send packet");
        self.need_acknowledge = false;
        println!("Client: Sent tics from {} to {}", start, end);
    }

    pub fn send_ticcmd(&mut self, ticcmd: &TicCmd, maketic: u32) {
        let mut diff = NetTicDiff::default();
        self.calculate_ticcmd_diff(ticcmd, &mut diff);

        let sendobj = &mut self.send_queue[maketic as usize % BACKUPTICS];
        sendobj.active = true;
        sendobj.seq = maketic;
        sendobj.time = Instant::now();
        sendobj.cmd = diff;

        let starttic = if maketic < self.settings.as_ref().unwrap().extratics as u32 {
            0
        } else {
            maketic - self.settings.as_ref().unwrap().extratics as u32
        };
        let endtic = maketic;

        self.send_tics(starttic, endtic);
    }

    fn calculate_ticcmd_diff(&self, ticcmd: &TicCmd, diff: &mut NetTicDiff) {
        diff.diff = 0;
        diff.cmd = *ticcmd;

        if self.last_ticcmd.forwardmove != ticcmd.forwardmove {
            diff.diff |= NET_TICDIFF_FORWARD;
        }

        if self.last_ticcmd.sidemove != ticcmd.sidemove {
            diff.diff |= NET_TICDIFF_SIDE;
        }

        if self.last_ticcmd.angleturn != ticcmd.angleturn {
            diff.diff |= NET_TICDIFF_TURN;
        }

        if self.last_ticcmd.buttons != ticcmd.buttons {
            diff.diff |= NET_TICDIFF_BUTTONS;
        }

        if self.last_ticcmd.consistancy != ticcmd.consistancy {
            diff.diff |= NET_TICDIFF_CONSISTANCY;
        }

        if ticcmd.chatchar != 0 {
            diff.diff |= NET_TICDIFF_CHATCHAR;
        } else {
            diff.cmd.chatchar = 0;
        }

        if self.last_ticcmd.lookfly != ticcmd.lookfly || ticcmd.arti != 0 {
            diff.diff |= NET_TICDIFF_RAVEN;
        } else {
            diff.cmd.arti = 0;
        }

        if self.last_ticcmd.buttons2 != ticcmd.buttons2 || ticcmd.inventory != 0 {
            diff.diff |= NET_TICDIFF_STRIFE;
        } else {
            diff.cmd.inventory = 0;
        }
    }

    fn advance_window(&mut self) {
        while self.recv_window[0].active {
            let mut ticcmds = [TicCmd::default(); NET_MAXPLAYERS];
            let window_start = self.recv_window_start;

            let window = self.recv_window[0].cmd;
            self.expand_full_ticcmd(&window, window_start, &mut ticcmds);

            // Call D_ReceiveTic or equivalent game state update function
            self.receive_tic(&ticcmds, &self.recv_window[0].cmd.playeringame);

            // Shift the window
            self.recv_window.rotate_left(1);
            self.recv_window[BACKUPTICS - 1] = NetServerRecv::default();
            self.recv_window_start += 1;

            println!(
                "Client: Advanced receive window to {}",
                self.recv_window_start
            );
        }
    }

    fn expand_full_ticcmd(
        &mut self,
        cmd: &NetFullTicCmd,
        seq: u32,
        ticcmds: &mut [TicCmd; NET_MAXPLAYERS],
    ) {
        let consoleplayer = self.settings.as_ref().unwrap().consoleplayer as usize;
        let drone = self.drone;
        let mut recvwindow_cmd_base = self.recvwindow_cmd_base.clone();

        for i in 0..NET_MAXPLAYERS {
            if i == consoleplayer && !drone {
                continue;
            }

            if cmd.playeringame[i] {
                let diff = &cmd.cmds[i];
                let mut base = recvwindow_cmd_base[i];
                NetClient::apply_ticcmd_diff(&mut base, diff, &mut ticcmds[i]);
                recvwindow_cmd_base[i] = ticcmds[i];
            }
        }

        self.recvwindow_cmd_base = recvwindow_cmd_base;
    }

    fn apply_ticcmd_diff(base: &mut TicCmd, diff: &NetTicDiff, result: &mut TicCmd) {
        *result = *base;

        if diff.diff & NET_TICDIFF_FORWARD != 0 {
            result.forwardmove = diff.cmd.forwardmove;
        }
        if diff.diff & NET_TICDIFF_SIDE != 0 {
            result.sidemove = diff.cmd.sidemove;
        }
        if diff.diff & NET_TICDIFF_TURN != 0 {
            result.angleturn = diff.cmd.angleturn;
        }
        if diff.diff & NET_TICDIFF_BUTTONS != 0 {
            result.buttons = diff.cmd.buttons;
        }
        if diff.diff & NET_TICDIFF_CONSISTANCY != 0 {
            result.consistancy = diff.cmd.consistancy;
        }
        if diff.diff & NET_TICDIFF_CHATCHAR != 0 {
            result.chatchar = diff.cmd.chatchar;
        } else {
            result.chatchar = 0;
        }
        if diff.diff & NET_TICDIFF_RAVEN != 0 {
            result.lookfly = diff.cmd.lookfly;
            result.arti = diff.cmd.arti;
        } else {
            result.arti = 0;
        }
        if diff.diff & NET_TICDIFF_STRIFE != 0 {
            result.buttons2 = diff.cmd.buttons2;
            result.inventory = diff.cmd.inventory;
        } else {
            result.inventory = 0;
        }

        *base = *result;
    }

    fn receive_tic(
        &self,
        ticcmds: &[TicCmd; NET_MAXPLAYERS],
        playeringame: &[bool; NET_MAXPLAYERS],
    ) {
        // TODO: This function should update the game state with the new ticcmds
        println!(
            "Client: Received tic data for {} players",
            playeringame.iter().filter(|&&p| p).count()
        );
    }

    fn check_resends(&mut self) {
        let now = Instant::now();
        let mut resend_start = -1;
        let mut resend_end = -1;
        let maybe_deadlocked = now.duration_since(self.gamedata_recv_time) > Duration::from_secs(1);

        for i in 0..BACKUPTICS {
            let recvobj = &mut self.recv_window[i];
            let need_resend =
                !recvobj.active && recvobj.resend_time.elapsed() > Duration::from_millis(300);

            if i == 0
                && !recvobj.active
                && recvobj.resend_time.elapsed() > Duration::from_secs(1)
                && maybe_deadlocked
            {
                let need_resend = true;
            }

            if need_resend {
                if resend_start < 0 {
                    resend_start = i as i32;
                }

                resend_end = i as i32;
            } else if resend_start >= 0 {
                println!(
                    "Client: Resend request timed out for {}-{}",
                    self.recv_window_start + resend_start as u32,
                    self.recv_window_start + resend_end as u32
                );

                self.send_resend_request(
                    self.recv_window_start + resend_start as u32,
                    self.recv_window_start + resend_end as u32,
                );
                resend_start = -1;
            }
        }

        if resend_start >= 0 {
            println!(
                "Client: Resend request timed out for {}-{}",
                self.recv_window_start + resend_start as u32,
                self.recv_window_start + resend_end as u32
            );
            self.send_resend_request(
                self.recv_window_start + resend_start as u32,
                self.recv_window_start + resend_end as u32,
            );
        }

        if self.need_acknowledge
            && now.duration_since(self.gamedata_recv_time) > Duration::from_millis(200)
        {
            println!(
                "Client: No game data received since {:?}: triggering ack",
                self.gamedata_recv_time
            );
            self.send_game_data_ack();
        }
    }

    fn run_bot(&mut self) {
        if self.state == ClientState::InGame && self.drone {
            let maketic = self.recv_window_start + BACKUPTICS as u32;

            let cmd = self.bot.tick();
            self.send_ticcmd(&cmd, maketic);
        }
    }

    pub fn disconnect(&mut self) {
        if !self.net_client_connected {
            return;
        }

        println!("Client: Beginning disconnect");
        self.send_disconnect();

        let start_time = Instant::now();

        while self.state != ClientState::Disconnected {
            if start_time.elapsed() > Duration::from_secs(5) {
                println!("Client: No acknowledgment of disconnect received");
                self.state = ClientState::WaitingStart;
                eprintln!("NET_CL_Disconnect: Timeout while disconnecting from server");
                break;
            }

            self.run();
            std::thread::sleep(Duration::from_millis(1));
        }

        println!("Client: Disconnect complete");
        self.shutdown();
    }

    fn send_disconnect(&self) {
        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::Disconnect as u16);
        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send disconnect packet");
    }

    pub fn get_settings(&self) -> Option<GameSettings> {
        if self.state != ClientState::InGame {
            return None;
        }
        self.settings.clone()
    }

    pub fn launch_game(&mut self) {
        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::Launch as u16);
        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send launch packet");
    }

    pub fn start_game(&mut self, settings: &GameSettings) {
        self.last_ticcmd = TicCmd::default();

        let mut packet = NetPacket::new();
        packet.write_u16(NetPacketType::GameStart as u16);
        packet.write_settings(settings);
        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send game start packet");
    }

    pub fn connect(&mut self, addr: SocketAddr, connect_data: ConnectData) -> bool {
        self.server_addr = addr;
        self.state = ClientState::Disconnected;
        self.reject_reason = Some("Unknown reason".to_string());

        self.net_local_wad_sha1sum
            .copy_from_slice(&connect_data.wad_sha1sum);
        self.net_local_deh_sha1sum
            .copy_from_slice(&connect_data.deh_sha1sum);
        self.net_local_is_freedoom = connect_data.is_freedoom != 0;

        self.net_client_connected = true;
        self.net_client_received_wait_data = false;

        let start_time = Instant::now();
        self.last_send_time = Instant::now() - Duration::from_secs(1);

        while self.state == ClientState::Disconnected {
            let now = Instant::now();

            if now.duration_since(self.last_send_time) > Duration::from_secs(1) {
                self.send_syn(&connect_data);
                self.last_send_time = now;
            }

            if now.duration_since(start_time) > Duration::from_secs(120) {
                self.reject_reason = Some("No response from server".to_string());
                break;
            }

            self.run();
            std::thread::sleep(Duration::from_millis(1));
        }

        if self.state == ClientState::WaitingLaunch {
            println!("Client: Successfully connected");

            self.reject_reason = None;
            self.drone = connect_data.drone != 0;

            true
        } else {
            println!("Client: Connection failed");

            self.shutdown();
            false
        }
    }

    fn send_syn(&self, data: &ConnectData) {
        let mut packet = NetPacket::new();

        packet.write_u16(NetPacketType::Syn as u16);
        packet.write_u32(NET_MAGIC_NUMBER);
        packet.write_string(env!("CARGO_PKG_VERSION"));
        packet.write_connect_data(data);
        packet.write_string(&self.player_name);

        packet
            .send(&self.socket, &self.server_addr)
            .expect("Failed to send SYN packet");
        println!("Client: SYN sent");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_initialization() {
        let client = NetClient::new("Player1".to_string(), false);
        assert_eq!(client.player_name, "Player1");
        assert_eq!(client.drone, false);
    }
}
