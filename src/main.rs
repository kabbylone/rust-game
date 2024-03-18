use std::cmp;

use tcod::colors::*;
use tcod::console::*;

use rand::Rng;

// Window macros
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20;

// Map macros
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color{r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 100};

// room macros
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

struct Tcod {
    root: Root,
    con: Offscreen,
}

//general struct to define objects inside the game
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x,  y, char, color}
    }

    //move by the given amount if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize]._blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    //set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

//a rectangle used for a room
#[derive(Clone, Copy, Debug)]
struct Rect {
    _x1: i32,
    _x2: i32,
    _y1: i32,
    _y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            _x1: x,
            _x2: x + w,
            _y1: y,
            _y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self._x1 + self._x2) / 2;
        let center_y = (self._y1 + self._y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        //returns true if this rectangle intersects with another one
        (self._x1 <= other._x2)
            && (self._x2 >= other._x1)
            && (self._y1 <= other._y2)
            && (self._y2 >= other._y1)
    }
}

// map tile properties
#[derive(Clone, Copy, Debug)] //automatically implements certain traits
struct Tile {
    _blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            _blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            _blocked: true,
            block_sight: true,
        }
    }
}

//2d array vec, vec of vecs of tiles
type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

fn create_room(room: Rect, map: &mut Map) {
    //go through the tiles in the rectangle and make them passable excluding walls
    for x in (room._x1 + 1)..room._x2 {
        for y in (room._y1 + 1)..room._y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    //horizontal tunnel. 'min()' and 'max()' are used in case 'x1 > x2'
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    //vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_map(player: &mut Object) -> Map {
    //fill the map with "blocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    //making random rooms
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random w and h
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // random position w/o going out of bounds
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            //this means there are no intersections, valid room

            //"paint" it ot the map's tiles
            create_room(new_room, &mut map);

            //center coords of the new room
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                //first room where the player starts
                player.x = new_x;
                player.y = new_y;
            } else {
                //all rooms after the first:
                //connect to the prev room with tunnel

                //center coords of the prev room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                //random bool
                if rand::random() {
                    //h then v
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    //v then h
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            //append rooms
            rooms.push(new_room);
        }
    }

    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    
    // go through all tiles, and set their bg color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }

    //draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }

    // overlaying the con window over the root window to block out unwanted screenspace
    blit(
        &tcod.con,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

// function that handles key inputs
fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    // imports
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);
    // match key with function
    match key {
        //alt+enter = fullscreen
        Key{
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen)
        }
        // escape = exit 
        Key{code:Escape, ..} => return true,

        // movement
        Key{code: Up, ..} => player.move_by(0, -1, game),
        Key{code: Down, ..} => player.move_by(0, 1, game),
        Key{code: Left, ..} => player.move_by(-1, 0, game),
        Key{code: Right, ..} => player.move_by(1, 0, game),

        // dont register other key inputs
        _ => {}
    }
    false
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    // setting up the window
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike game")
        .init();

    //offscreen window to create effects over root
    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod {root, con};

    // player object
    let player = Object::new(0, 0, '@', WHITE);

    // NPC object
    let npc = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', YELLOW);

    //list of objects
    let mut objects = [player, npc];

    let game = Game {
        //generate map (not drawn on the screen)
        map: make_map(&mut objects[0]),
    };

    // game setup loop
    while !tcod.root.window_closed() {
        //clear the prev frame
        tcod.con.clear();

        for object in &objects {
            object.draw(&mut tcod.con);
        }

        //render the screen
        render_all(&mut tcod, &game, &objects);

        tcod.root.flush(); 

        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
