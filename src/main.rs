use macroquad::prelude::*;
use std::{borrow::Cow, net::{Ipv4Addr, SocketAddrV4, UdpSocket}};


struct World {
    local_player: Paddle,
    network_player: Paddle,
    ball: Ball,
}

impl World {
    fn new () -> Self {
        World {
            local_player: Paddle {
                x: 20.,
                y: 100.0,
                width: 20.0,
                height: 100.0,
                dx: 0.,
                dy: 0.,
            },
            network_player: Paddle {
                x: screen_width() - 40.,
                y: 100.,
                width: 20.,
                height: 100.,
                dx: 0.,
                dy: 0.,
            },
            ball: Ball {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
                dx: 0.,
                dy: 0.,
            },
        }
    }
}

struct ControlSystem;

impl ControlSystem {
    pub fn update_local_player(paddle:&mut Paddle) {
        if is_key_down(KeyCode::Up) {
            paddle.y += 4.;
        }
        else if is_key_down(KeyCode::Down) {
            paddle.y -= 4.;
        }
    }
}


struct PhysicsSystem;
impl PhysicsSystem {
    fn new() -> Self {
        PhysicsSystem
    }

    fn collision(ball: Ball, players: &Vec<Box<dyn GameObject>>) {
        let collision_player = players.iter()
            .filter(|p| p.get_position().0 < ball.x + ball.width)
            .filter(|p| p.get_position().0 + p.get_size().0 > ball.x);

        if collision_player.count() > 0 {
            // ball.dx = -ball.dx;
        }
    }
}

struct RenderSystem {
    objects: Vec<Box<dyn GameObject>>,
}

impl RenderSystem {
    fn new() -> Self {
        RenderSystem {
            objects: Vec::new(),
        }
    }

    fn add_objects(&mut self, objects: Vec<Box<dyn GameObject>>) {
        self.objects.extend(objects);
    }

    fn render(&self) {
        self.objects.iter().for_each(|object| object.draw());
    }
    
}

struct NetworkSystem {
    buf: [u8; 1024],
    socket: UdpSocket,
    client_addr: SocketAddrV4,
    server_addr: SocketAddrV4,
}

impl NetworkSystem {
    fn new(client_addr: SocketAddrV4) -> Self {
        NetworkSystem {
            buf: [0; 1024],
            client_addr,
            socket: UdpSocket::bind(client_addr).unwrap(),
            server_addr: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080)
        }
    }

    fn connect(&mut self) -> Cow<'_, str> {
        let buf = "join".as_bytes();
        let _  = self.socket.send_to(buf, self.server_addr);
        let (size, _source) = self.socket.recv_from(&mut self.buf).expect("Failed to connect to the server");
        String::from_utf8_lossy(&self.buf[..size])
    }

    fn send(&self, data: Vec<u8>) {
        // send data
    }

    fn receive(&self) -> Vec<u8> {
        // receive data
        Vec::new()
    }
}

trait GameObject {
    fn draw(&self);
    fn get_position(&self) -> (f32, f32);
    fn get_size(&self) -> (f32, f32);
    fn get_velocity(&self) -> (f32, f32);
    fn update(&self);
}

impl GameObject for Paddle {
    fn draw(&self) {
        draw_rectangle(self.x, self.y, self.width, self.height, WHITE);
    }

    fn update(&self) {
        // do nothing
    }

    fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    
    fn get_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn get_velocity(&self) -> (f32, f32) {
        (self.dx, self.dy)
    }
}

struct Paddle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    dx: f32,
    dy: f32,
}

struct Ball {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    dx: f32,
    dy: f32,
}


#[macroquad::main("Pong")]
async fn main() -> std::io::Result<()> {
    let client_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let mut network_system = NetworkSystem::new(client_addr); 
    let res =  network_system.connect();

    let left_paddle = Paddle {
        x: 20.,
        y: 100.0,
        width: 20.0,
        height: 100.0,
        dx: 0.,
        dy: 0.,
    };

    let right_paddle = Paddle {
        x: screen_width() - 40.,
        y: 100.,
        width: 20.,
        height: 100.,
        dx: 0.,
        dy: 0.,
    };

    let mut control_system = ControlSystem;


    let mut pos_x = 10.;
    let ball_dx = 4.;
    let mut ball_x = 0.; 
    

    loop {
        ball_x += ball_dx;
        control_system.update_local_player();


        clear_background(BLACK);

        //draw_rectangle(20., 100.0 - pos_x, 20.0, 100.0, WHITE);
        //draw_rectangle(screen_width() - 40., 100.0 - pos_x, 20.0, 100.0, WHITE);
        draw_circle(screen_width() / 2. + ball_x , screen_height() / 2.0, 15.0, WHITE);

        next_frame().await
    }
}

