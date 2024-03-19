use std::cmp;

use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

use rand::Rng;

// Window macros
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20;

// Map macros
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color{r: 0, g: 0, b: 100};
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50};
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 100};
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b:50};

//player
const PLAYER: usize = 0;

// room macros
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_ROOM_MONSTERS: i32 = 3;

//fov macros
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

//general struct to define objects inside the game
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
        }
    }

    //set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x : i32, y: i32) {
        self.x = x;
        self.y = y;
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
    _explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            _blocked: false,
            _explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            _blocked: true,
            _explored: false,
            block_sight: true,
        }
    }
}

//2d array vec, vec of vecs of tiles
type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

//move by the given amount if the destination is not blocked
fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    //first test the map tile
    if map[x as usize][y as usize]._blocked {
        return true;
    }
    //now check for any blocking objects
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
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

fn make_map(objects: &mut Vec<Object>) -> Map {
    //fill the map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
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

            //add some content to this room, such as monsters
            place_objects(new_room, &map, objects);

            //center coords of the new room
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                //first room where the player starts
                objects[PLAYER].set_pos(new_x, new_y);
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

fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    //choose random no of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        //choose random spot for this monster
        let x = rand::thread_rng().gen_range(room._x1 + 1, room._x2);
        let y = rand::thread_rng().gen_range(room._y1 + 1, room._y2);

        //only place if the tile is not blocked
        if !is_blocked(x, y, map, objects) {
            let mut monster = if rand::random::<f32>() < 0.8 { //80% chance of orc
                //create an orc
                Object::new(x, y, 'O', "orc", DESATURATED_GREEN, true)
            } else {
                //create a troll
                Object::new(x, y, 'T', "troll", DARKER_GREEN, true)
            };
            
            monster.alive = true;
            objects.push(monster);
        }
    }
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {

    if fov_recompute {
        //recompute FOV if needed (the player moved or any other event)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }
    
    // go through all tiles, and set their bg color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match(visible, wall) {
                //outside of field of view
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                //inside fov
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize]._explored;
            if visible {
                //since it's visible, explore it
                *explored = true;
            }
            if *explored {
                //show explored tiles only (any visible tile is explored already)
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }

    //draw all objects in the list
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
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

fn player_move_or_attack(dx: i32, dy: i32, game: &Game, objects: &mut [Object]) {
    //coords the player is attacking/moving to
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    //try to find an attackable object
    let target_id = objects.iter().position(|object| object.pos() == (x, y));

    //attack if target found, otherwise move
    match target_id {
        Some(target_id) => {
            println!(
                "The {} laughs at your puny efforts to attack him!",
                objects[target_id].name
            );
        }
        None => {
            move_by(PLAYER, dx, dy, &game.map, objects);
        }
    }
}

// function that handles key inputs
fn handle_keys(tcod: &mut Tcod, game: &Game, objects: &mut Vec<Object>) -> PlayerAction {
    // imports
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let key = tcod.root.wait_for_keypress(true);
    let player_alive = objects[PLAYER].alive;
    // match key with function
    match (key, key.text(), player_alive) {
        //alt+enter = fullscreen
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }

        // escape = exit 
        (Key { code: Escape, ..}, _, _) => Exit,

        // movement
        (Key{code: Up, ..}, _, true) => {
            player_move_or_attack(0, -1, game, objects);
            TookTurn
        }
        (Key{code: Down, ..}, _, true) => {
            player_move_or_attack(0, 1, game, objects);
            TookTurn
        }
        (Key{code: Left, ..}, _, true) => {
            player_move_or_attack(-1, 0, game, objects);
            TookTurn
        }
        (Key{code: Right, ..}, _, true) => {
            player_move_or_attack(1, 0, game, objects);
            TookTurn
        }

        // dont register other key inputs
        _ => DidntTakeTurn
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
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

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    // player object
    let player = Object::new(0, 0, '@', "player", WHITE, true);

    //list of objects with just the player
    let mut objects = vec![player];

    let mut game = Game {
        //generate map (not drawn on the screen)
        map: make_map(&mut objects),
    };

    //populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set (
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize]._blocked,
            );
        }
    }

    //force FOC "recompute" first time through the fame loop
    let mut previous_player_position = (-1, -1);

    // game setup loop
    while !tcod.root.window_closed() {
        //clear the prev frame
        tcod.con.clear();

        //render the screen
        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(&mut tcod, &mut game, &objects, fov_recompute);

        tcod.root.flush(); 

        previous_player_position = objects[PLAYER].pos();
        let player_action = handle_keys(&mut tcod, &game, &mut objects);
        if player_action == PlayerAction::Exit {
            break;
        }

        //let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for object in &objects {
                //only if object is not player
                if (object as *const _) != (&objects[PLAYER] as *const _) {
                    println!("The {} growls!", object.name);
                }
            }
        }
    }
}
