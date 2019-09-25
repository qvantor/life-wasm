use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

trait Renderable {
    fn render(&self, _context: &web_sys::CanvasRenderingContext2d);
}

#[derive(Debug)]
pub struct Grid {
    cell: i16,
    size: i16,
}

impl Renderable for Grid {
    fn render(&self, _context: &web_sys::CanvasRenderingContext2d) {
        let max_size = (self.size * self.cell).into();
        for i in 0..self.size {
            let position = (i * self.cell).into();

            _context.move_to(0.0, position);
            _context.line_to(max_size, position);
            _context.close_path();

            _context.move_to(position, 0.0);
            _context.line_to(position, max_size);
            _context.close_path();
        }
        _context.move_to(0.0, max_size);
        _context.line_to(max_size, max_size);
        _context.close_path();

        _context.move_to(max_size, 0.0);
        _context.line_to(max_size, max_size);
        _context.close_path();
        _context.stroke();
    }
}

#[derive(Debug)]
struct Game {
    grid: Grid,
    state: Vec<Vec<bool>>,
}

impl Renderable for Game {
    fn render(&self, _context: &web_sys::CanvasRenderingContext2d) {
        _context.clear_rect(0.0, 0.0, 1000.0, 1000.0);
        for (col_num, col) in self.state.iter().enumerate() {
            for row_num in 0..col.len() {
                if col[row_num] {
                    _context.fill_rect(
                        col_num as f64 * self.grid.cell as f64,
                        row_num as f64 * self.grid.cell as f64,
                        self.grid.cell as f64,
                        self.grid.cell as f64);
                }
            }
        }
        self.grid.render(_context);
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

impl Game {
    fn generate_row(&self) -> Vec<bool> {
        let mut row = vec![];
        for _i in 0..self.grid.size {
            row.push(rand::random());
        }
        return row;
    }
    fn generate_initial_state(&self) -> Vec<Vec<bool>> {
        let mut initial_store = vec![];
        for _col in 0..self.grid.size {
            initial_store.push(self.generate_row());
        }
        return initial_store;
    }
    fn start(&mut self, _context: &web_sys::CanvasRenderingContext2d) -> &mut Game {
        self.state = self.generate_initial_state();
        self.render(_context);
        self
    }
    fn calc_tick(&self) -> Vec<Vec<bool>> {
        let mut new_state = vec![];
        for (col_num, col) in self.state.iter().enumerate() {
            new_state.push(col.to_vec());
            for row_num in 0..col.len() {
                let nebour_count = self.get_nebour_count(row_num as i16, col_num as i16);
                if col[row_num] {
                    if nebour_count < 2 {
                        new_state[col_num][row_num] = false;
                    } else if nebour_count > 3 {
                        new_state[col_num][row_num] = false;
                    }
                } else if nebour_count == 3 {
                    new_state[col_num][row_num] = true;
                }
            }
        }
        new_state
    }
    fn get_nebour_count(&self, i: i16, j: i16) -> i8 {
        let mut count: i8 = 0;
        for ni in (i - 1)..=(i + 1) {
            for nj in (j - 1)..=(j + 1) {
                if ni < 0 || ni >= self.grid.size { continue; }
                if nj < 0 || nj >= self.grid.size { continue; }
                if ni == i && nj == j { continue; }
                if self.state[nj as usize][ni as usize] {
                    count += 1;
                }
            }
        }
        count
    }
    fn tick(&mut self, _context: &web_sys::CanvasRenderingContext2d) -> bool {
        self.state = self.calc_tick();
        self.render(_context);
        true
    }
}


#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let performance = window.performance().expect("performance should be available");

    const FPS: f64 = 15.0;
    let mut game = Game {
        grid: Grid {
            cell: 15,
            size: 50,
        },
        state: vec![],
    };

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    &context.translate(10.0, 10.0);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("red"));

    game.start(&context);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut time: f64 = 0.0;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        request_animation_frame(f.borrow().as_ref().unwrap());
        if performance.now() - time > 1000.0 / FPS {
            time = performance.now();
            game.tick(&context);
        }
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
}
