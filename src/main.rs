use tcod::colors::*;
use tcod::console::*;

// Window macros
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20;

// Map macros
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color{r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 100};

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

fn make_map() -> Map {
    //fill the map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    //place 2 pillars to test the map
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    //draw all objects in the list
    for object in objects {
        object.draw(&mut tcod.con);
    }

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
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);

    // NPC object
    let npc = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', YELLOW);

    //list of objects
    let mut objects = [player, npc];

    let game = Game {
        //generate map (not drawn on the screen)
        map: make_map(),
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
