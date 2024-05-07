use macroquad::prelude::*;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    slice::Iter,
};

struct Entity {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    player: bool,
}

impl ToString for Entity {
    fn to_string(&self) -> String {
        format!("{} {} {} {}", self.x, self.y, self.width, self.height)
    }
}

struct World {
    entities: Vec<Entity>,
}

impl World {
    fn new() -> Self {
        World {
            entities: Vec::new(),
        }
    }

    fn get_entities(&self) -> Iter<Entity> {
        self.entities.iter()
    }

    fn get_player(&mut self) -> &mut Entity {
        self.entities
            .iter_mut()
            .find(|entity| entity.player)
            .unwrap()
    }

    fn get_opponent(&mut self) -> &mut Entity {
        self.entities
            .iter_mut()
            .find(|entity| !entity.player)
            .unwrap()
    }

    fn get_ball(&mut self) -> &mut Entity {
        self.entities
            .iter_mut()
            .find(|entity| entity.width == entity.height)
            .unwrap()
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
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
        let socket = UdpSocket::bind(client_addr).unwrap();
        NetworkSystem {
            buf: [0; 1024],
            client_addr,
            socket,
            server_addr: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080),
        }
    }

    fn parse(&mut self, size: usize) -> Vec<f32> {
        println!("size: {}", size);
        std::str::from_utf8(&self.buf[..size])
            .unwrap()
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok())
            .collect()
    }

    fn connect(&mut self) -> Entity {
        let buf = "join".as_bytes();
        let _ = self.socket.send_to(buf, self.server_addr);
        let (size, _source) = self
            .socket
            .recv_from(&mut self.buf)
            .expect("Failed to connect to the server");
        let floats: Vec<f32> = self.parse(size);

        if floats.len() != 4 {
            panic!("Failed to connect to the server");
        }

        let (x, y, width, height) = (floats[0], floats[1], floats[2], floats[3]);

        self.socket
            .set_nonblocking(true)
            .expect("Failed to set non-blocking mode");
        println!("Connected to the server");
        Entity {
            x,
            y,
            width,
            height,
            player: true,
        }
    }

    fn send(&mut self, data: Vec<u8>) {
        let _ = self.socket.send_to(&data, self.server_addr);
    }

    fn listen(&mut self) -> Result<(usize, SocketAddr), std::io::Error> {
        self.socket.recv_from(&mut self.buf)
    }
}

struct RenderSystem;

impl RenderSystem {
    fn render(&self, world: &World) {
        world.get_entities().for_each(|entity| {
            draw_rectangle(entity.x, entity.y, entity.width, entity.height, WHITE);
        })
    }
}
struct ControlSystem;

impl ControlSystem {
    fn movement(&self, world: &mut World, network_system: &mut NetworkSystem) {
        if is_key_down(KeyCode::S) {
            world.get_player().y += 4.0;
            //network_system.send(world.get_player().to_string().as_bytes().to_vec());
        }
        if is_key_down(KeyCode::W) {
            world.get_player().y -= 4.0;
            //network_system.send(world.get_player().to_string().as_bytes().to_vec());
        }
    }

    fn add_ball(&self, world: &mut World, floats: &Vec<f32>) {
        if world.get_entities().count() == 2 && floats.len() == 8 {
            println!("Adding ball");
            world.add_entity(Entity {
                x: floats[4],
                y: floats[5],
                width: floats[6],
                height: floats[7],
                player: false,
            });
        }
    }

    fn add_opponent(&self, world: &mut World, floats: &Vec<f32>) {
        if world.get_entities().count() == 1 && floats.len() > 4 {
            println!("Adding opponent");
            let (x, y, width, height) = (floats[0], floats[1], floats[2], floats[3]);

            world.add_entity(Entity {
                x,
                y,
                width,
                height,
                player: false,
            });
        }
    }

    fn update_opponent(&self, world: &mut World, floats: &Vec<f32>) {
        if world.get_entities().count() >= 2 && floats.len() >= 4 {
            world.get_opponent().x = floats[0];
            world.get_opponent().y = floats[1];
        }
    }

    fn update_ball(&self, world: &mut World, floats: &Vec<f32>) {
        if floats.len() == 8 {
            world.get_ball().x = floats[4];
            world.get_ball().y = floats[5];
        }
    }

    fn update_ball_locally(&self, world: &mut World) {
        if world.get_entities().count() > 2 {
            world.get_ball().x += 70. * get_frame_time();
            //world.get_ball().y += 1.;
        }
    }
}

#[macroquad::main("Pong")]
async fn main() -> std::io::Result<()> {
    let client_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let mut network_system = NetworkSystem::new(client_addr);
    let res = network_system.connect();

    let mut world = World::new();
    world.add_entity(res);
    let render_system = RenderSystem;
    let control_system = ControlSystem;

    loop {
        clear_background(BLACK);

        control_system.movement(&mut world, &mut network_system);
        render_system.render(&world);
        network_system.send(world.get_player().to_string().as_bytes().to_vec());

        match network_system.listen() {
            Ok((usize, _source)) => {
                let floats = network_system.parse(usize);
                println!("Received: {:?}", floats);
                control_system.add_opponent(&mut world, &floats);
                control_system.add_ball(&mut world, &floats);
                control_system.update_opponent(&mut world, &floats);
                control_system.update_ball(&mut world, &floats);
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    eprintln!("Error: {:?}", e);
                }
            }
        };

        next_frame().await
    }
}
